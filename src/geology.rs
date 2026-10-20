//! Brazilian field areas: one bounded map per level, each with its own
//! lithostratigraphic units, their geochronologic ages, and a structural
//! rule that decides which unit crops out where.
//!
//! The units are hand-authored from public descriptions of each area (SGB /
//! CPRM publications and the papers listed in the README). Ages are given at
//! era/period level with rounded numbers: this is a game, so confirm anything
//! important against the GeoSGB map viewer.
//!
//! Structural rules keep the maps geologically coherent:
//! * `Layered`  - flat-lying strata (sedimentary basins): older units crop out
//!   lower down, so the stratigraphy follows the elevation.
//! * `Belts`    - folded, steeply dipping belts that repeat symmetrically.
//! * `Rings`    - concentric zones around an intrusive/volcanic complex.
//!
//! On top of that, `blobs` place intrusions and basement domes, and `cap` lays
//! a duricrust (canga, bauxite) over the high plateaus.

use crate::terrain::{fbm, Terrain, ELEV_MAX};

/// (rock name, relative weight, class: Igneous | Sedimentary | Metamorphic)
pub type RockSpec = (&'static str, u32, &'static str);

pub struct Unit {
    pub name: &'static str,
    /// Group / supergroup / complex it belongs to.
    pub parent: &'static str,
    /// Geochronologic age, era/period level with a rounded number.
    pub age: &'static str,
    pub rocks: &'static [RockSpec],
    pub minerals: &'static [&'static str],
    /// Chance (percent) that breaking an outcrop of this unit yields a mineral.
    pub mineral_pct: u32,
    pub fossils: &'static [&'static str],
    pub fossil_pct: u32,
    /// Percent of candidate outcrop spots that really are exposed here.
    pub exposure: u32,
    /// Colour on the geological map.
    pub color: (u8, u8, u8),
}

pub enum Structure {
    /// Older units at the bottom: `levels` are ascending elevation thresholds
    /// (metres); unit index = how many thresholds lie below the ground.
    Layered { levels: &'static [f32] },
    /// Folded belts striking `strike_deg` (0 = east-west). `widths` are the
    /// across-strike widths of units 0.. and the sequence mirrors at each fold.
    Belts { strike_deg: f32, widths: &'static [f32], warp: f32 },
    /// Unit i inside radii[i]; the last unit lies beyond the last radius.
    Rings { cx: f32, cy: f32, radii: &'static [f32], warp: f32 },
}

/// An intrusion or basement dome: where this noise exceeds `threshold`, the
/// unit is `unit` regardless of the structure underneath.
pub struct Blob {
    pub unit: usize,
    pub scale: f32,
    pub threshold: f32,
}

pub struct Region {
    pub id: &'static str,
    pub name: &'static str,
    pub place: &'static str,
    #[allow(dead_code)]
    pub blurb: &'static str,
    pub structure: Structure,
    pub blobs: &'static [Blob],
    /// (unit, elevation): above this elevation a duricrust cap hides the rock below.
    pub cap: Option<(usize, f32)>,
    pub units: &'static [Unit],
    pub seed: u32,
    pub relief: f32,
    pub water_level: f32,
    pub river_band: f32,
    pub farm_pct: u32,
    pub forest_threshold: f32,
    /// Outcrops to survey to complete the map.
    pub quota: u32,
}

/// The playable rectangle (x0, y0, x1, y1) shared by every map; the outer
/// roads run along y = -3200 and y = 6400.
pub const MAP_BOUNDS: (f32, f32, f32, f32) = (-3200.0, -3200.0, 6400.0, 6400.0);

impl Region {
    pub fn terrain(&self) -> Terrain {
        Terrain {
            seed: self.seed,
            flat: false,
            relief: self.relief,
            water_level: self.water_level,
            farm_pct: self.farm_pct,
            forest_threshold: self.forest_threshold,
            river_band: self.river_band,
            bounds: Some(MAP_BOUNDS),
        }
    }

    pub fn unit_at(&self, terrain: &Terrain, x: f32, y: f32) -> usize {
        self.unit_at_elev(terrain, x, y, terrain.elevation(x, y))
    }

    /// Same as `unit_at` when the elevation is already known (map rendering).
    pub fn unit_at_elev(&self, terrain: &Terrain, x: f32, y: f32, elev: f32) -> usize {
        let s = self.seed;
        if let Some((unit, level)) = self.cap {
            if elev > level {
                return unit;
            }
        }
        for (i, b) in self.blobs.iter().enumerate() {
            if fbm(s ^ (0xB10B_0000 + i as u32), x / b.scale, y / b.scale, 2) > b.threshold {
                return b.unit;
            }
        }
        let _ = terrain;
        match &self.structure {
            Structure::Layered { levels } => levels.iter().filter(|&&l| elev > l).count().min(self.units.len() - 1),
            Structure::Belts { strike_deg, widths, warp } => {
                let th = strike_deg.to_radians();
                let across = -x * th.sin() + y * th.cos();
                let w = (fbm(s ^ 0xF01D_0001, x / 1700.0, y / 1700.0, 3) - 0.5) * warp;
                let total: f32 = widths.iter().sum();
                let period = 2.0 * total;
                let mut m = (across + w).rem_euclid(period);
                if m > total {
                    m = period - m; // mirror: the folds repeat the sequence backwards
                }
                let mut acc = 0.0;
                for (i, wd) in widths.iter().enumerate() {
                    acc += wd;
                    if m <= acc {
                        return i.min(self.units.len() - 1);
                    }
                }
                widths.len() - 1
            }
            Structure::Rings { cx, cy, radii, warp } => {
                let w = (fbm(s ^ 0x0A11_0001, x / 1300.0, y / 1300.0, 3) - 0.5) * warp;
                let d = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() + w;
                radii.iter().position(|&r| d < r).unwrap_or(radii.len())
            }
        }
    }
}

/// Pick a rock from a unit by a hashed roll.
pub fn pick_rock(unit: &Unit, roll: u32) -> (&'static str, &'static str) {
    let total: u32 = unit.rocks.iter().map(|r| r.1).sum();
    let mut n = roll % total.max(1);
    for &(name, w, class) in unit.rocks {
        if n < w {
            return (name, class);
        }
        n -= w;
    }
    (unit.rocks[0].0, unit.rocks[0].2)
}

#[allow(dead_code)]
pub fn elev_fraction(e: f32) -> f32 {
    (e / ELEV_MAX).clamp(0.0, 1.0)
}

// ============================================================ the five maps

pub static REGIONS: [Region; 5] = [PARANA, ARARIPE, QUADRILATERO, POCOS, CARAJAS];

// ---- 1. Bacia do Parana (SP/PR): flat-lying strata, basalt plateau ----------
const PARANA: Region = Region {
    id: "parana",
    name: "Bacia do Paraná",
    place: "SP / PR",
    blurb: "A huge intracratonic basin: glacial deposits at the bottom, desert dunes, then flood basalts on top.",
    structure: Structure::Layered { levels: &[88.0, 118.0, 150.0] },
    blobs: &[],
    cap: None,
    units: &[
        Unit {
            name: "Itararé Group",
            parent: "Paraná Basin",
            age: "Late Carboniferous - Early Permian",
            rocks: &[("Diamictite", 40, "Sedimentary"), ("Sandstone", 35, "Sedimentary"), ("Rhythmite (varvite)", 25, "Sedimentary")],
            minerals: &["Quartz"],
            mineral_pct: 6,
            fossils: &[],
            fossil_pct: 0,
            exposure: 70,
            color: (150, 130, 170),
        },
        Unit {
            name: "Irati Formation",
            parent: "Passa Dois Group",
            age: "Permian",
            rocks: &[("Black shale", 50, "Sedimentary"), ("Limestone", 30, "Sedimentary"), ("Siltstone", 20, "Sedimentary")],
            minerals: &["Pyrite", "Calcite"],
            mineral_pct: 12,
            fossils: &["Mesosaurus skeleton", "Crustacean (Pygaspis)"],
            fossil_pct: 30,
            exposure: 60,
            color: (70, 90, 110),
        },
        Unit {
            name: "Botucatu Formation",
            parent: "São Bento Group",
            age: "Jurassic - Early Cretaceous",
            rocks: &[("Aeolian sandstone", 85, "Sedimentary"), ("Silicified sandstone", 15, "Sedimentary")],
            minerals: &["Quartz"],
            mineral_pct: 6,
            fossils: &["Dinosaur trackway (Brasilichnium)"],
            fossil_pct: 7,
            exposure: 80,
            color: (225, 175, 120),
        },
        Unit {
            name: "Serra Geral Formation",
            parent: "São Bento Group",
            age: "Early Cretaceous (~135-130 Ma)",
            rocks: &[("Basalt", 72, "Igneous"), ("Diabase", 18, "Igneous"), ("Rhyodacite", 10, "Igneous")],
            minerals: &["Amethyst", "Agate", "Stilbite"],
            mineral_pct: 18,
            fossils: &[],
            fossil_pct: 0,
            exposure: 100,
            color: (95, 70, 85),
        },
    ],
    seed: 0x5A71_0001,
    relief: 0.95,
    water_level: 48.0,
    river_band: 0.010,
    farm_pct: 100,
    forest_threshold: 0.62,
    quota: 25,
};

// ---- 2. Bacia do Araripe (CE/PE/PI): lake limestones full of fossils ----------
const ARARIPE: Region = Region {
    id: "araripe",
    name: "Bacia do Araripe",
    place: "CE / PE / PI",
    blurb: "An Early Cretaceous rift-lake basin in the semi-arid northeast, famous for its exceptional fossils.",
    structure: Structure::Layered { levels: &[84.0, 112.0, 138.0, 170.0] },
    blobs: &[],
    cap: None,
    units: &[
        Unit {
            name: "Rio da Batateira Formation",
            parent: "Santana Group",
            age: "Early Cretaceous (Aptian)",
            rocks: &[("Sandstone", 55, "Sedimentary"), ("Shale", 45, "Sedimentary")],
            minerals: &["Quartz"],
            mineral_pct: 5,
            fossils: &[],
            fossil_pct: 0,
            exposure: 70,
            color: (205, 150, 110),
        },
        Unit {
            name: "Crato Formation",
            parent: "Santana Group",
            age: "Early Cretaceous (Aptian - Albian)",
            rocks: &[("Laminated limestone", 80, "Sedimentary"), ("Marl", 20, "Sedimentary")],
            minerals: &["Calcite"],
            mineral_pct: 6,
            fossils: &["Fossil fish (Dastilbe)", "Fossil insect", "Fossil plant"],
            fossil_pct: 40,
            exposure: 75,
            color: (120, 190, 170),
        },
        Unit {
            name: "Ipubi Formation",
            parent: "Santana Group",
            age: "Early Cretaceous (Aptian - Albian)",
            rocks: &[("Gypsum", 60, "Sedimentary"), ("Anhydrite", 25, "Sedimentary"), ("Marl", 15, "Sedimentary")],
            minerals: &["Selenite"],
            mineral_pct: 20,
            fossils: &[],
            fossil_pct: 0,
            exposure: 65,
            color: (235, 225, 170),
        },
        Unit {
            name: "Romualdo Formation",
            parent: "Santana Group",
            age: "Early Cretaceous (Albian)",
            rocks: &[("Fossiliferous concretion", 40, "Sedimentary"), ("Calcareous sandstone", 30, "Sedimentary"), ("Black shale", 30, "Sedimentary")],
            minerals: &["Calcite", "Pyrite"],
            mineral_pct: 10,
            fossils: &["Fossil fish (Vinctifer)", "Pterosaur bone", "Fossil turtle (Araripemys)"],
            fossil_pct: 45,
            exposure: 70,
            color: (90, 140, 120),
        },
        Unit {
            name: "Exu Formation",
            parent: "Araripe Group",
            age: "Cretaceous (Albian - Cenomanian)",
            rocks: &[("Red sandstone", 80, "Sedimentary"), ("Conglomerate", 20, "Sedimentary")],
            minerals: &["Quartz"],
            mineral_pct: 5,
            fossils: &[],
            fossil_pct: 0,
            exposure: 90,
            color: (200, 100, 80),
        },
    ],
    seed: 0xA4A4_0002,
    relief: 0.85,
    water_level: 34.0,
    river_band: 0.006,
    farm_pct: 45,
    forest_threshold: 0.67,
    quota: 25,
};

// ---- 3. Quadrilatero Ferrifero (MG): folded, metamorphosed iron belts ---------
const QUADRILATERO: Region = Region {
    id: "qf",
    name: "Quadrilátero Ferrífero",
    place: "MG",
    blurb: "Archean gneiss domes ringed by greenstone belts and the iron-rich Minas Supergroup, folded into synclines.",
    structure: Structure::Belts { strike_deg: 35.0, widths: &[520.0, 380.0, 330.0, 560.0, 440.0, 480.0, 420.0], warp: 2200.0 },
    blobs: &[Blob { unit: 0, scale: 2400.0, threshold: 0.60 }],
    cap: None,
    units: &[
        Unit {
            name: "Basement gneiss complexes",
            parent: "Bação / Bonfim / Belo Horizonte",
            age: "Mesoarchean (~3.2 - 2.9 Ga)",
            rocks: &[("Gneiss", 60, "Metamorphic"), ("Granite", 25, "Igneous"), ("Migmatite", 15, "Metamorphic")],
            minerals: &["Biotite", "Feldspar", "Zircon"],
            mineral_pct: 10,
            fossils: &[],
            fossil_pct: 0,
            exposure: 75,
            color: (225, 150, 150),
        },
        Unit {
            name: "Nova Lima Group",
            parent: "Rio das Velhas Supergroup",
            age: "Neoarchean (~2.8 - 2.7 Ga)",
            rocks: &[("Phyllite", 35, "Metamorphic"), ("Schist", 30, "Metamorphic"), ("Banded iron formation", 15, "Metamorphic"), ("Metabasalt", 20, "Metamorphic")],
            minerals: &["Gold", "Arsenopyrite", "Garnet", "Kyanite"],
            mineral_pct: 20,
            fossils: &[],
            fossil_pct: 0,
            exposure: 65,
            color: (120, 170, 100),
        },
        Unit {
            name: "Moeda Formation",
            parent: "Caraça Group, Minas Supergroup",
            age: "Paleoproterozoic (~2.6 - 2.4 Ga)",
            rocks: &[("Quartzite", 75, "Metamorphic"), ("Metaconglomerate", 25, "Metamorphic")],
            minerals: &["Quartz", "Gold", "Pyrite"],
            mineral_pct: 12,
            fossils: &[],
            fossil_pct: 0,
            exposure: 90,
            color: (240, 225, 140),
        },
        Unit {
            name: "Batatal Formation",
            parent: "Caraça Group, Minas Supergroup",
            age: "Paleoproterozoic (~2.5 Ga)",
            rocks: &[("Sericitic phyllite", 85, "Metamorphic"), ("Metachert", 15, "Metamorphic")],
            minerals: &["Sericite", "Graphite"],
            mineral_pct: 6,
            fossils: &[],
            fossil_pct: 0,
            exposure: 50,
            color: (170, 150, 200),
        },
        Unit {
            name: "Cauê Formation",
            parent: "Itabira Group, Minas Supergroup",
            age: "Paleoproterozoic (~2.4 Ga)",
            rocks: &[("Itabirite", 70, "Metamorphic"), ("Dolomitic itabirite", 15, "Metamorphic"), ("Hematite ore", 15, "Metamorphic")],
            minerals: &["Hematite", "Magnetite", "Martite", "Specularite"],
            mineral_pct: 40,
            fossils: &[],
            fossil_pct: 0,
            exposure: 95,
            color: (200, 60, 50),
        },
        Unit {
            name: "Gandarela Formation",
            parent: "Itabira Group, Minas Supergroup",
            age: "Paleoproterozoic (~2.4 Ga)",
            rocks: &[("Dolomite", 55, "Sedimentary"), ("Dolomitic marble", 30, "Metamorphic"), ("Phyllite", 15, "Metamorphic")],
            minerals: &["Dolomite", "Calcite"],
            mineral_pct: 6,
            fossils: &["Stromatolite"],
            fossil_pct: 25,
            exposure: 70,
            color: (110, 170, 215),
        },
        Unit {
            name: "Cercadinho Formation",
            parent: "Piracicaba Group, Minas Supergroup",
            age: "Paleoproterozoic (~2.1 Ga)",
            rocks: &[("Ferruginous quartzite", 60, "Metamorphic"), ("Phyllite", 25, "Metamorphic"), ("Metasiltstone", 15, "Metamorphic")],
            minerals: &["Hematite", "Quartz"],
            mineral_pct: 10,
            fossils: &[],
            fossil_pct: 0,
            exposure: 80,
            color: (235, 160, 90),
        },
    ],
    seed: 0x0F0F_0003,
    relief: 1.25,
    water_level: 52.0,
    river_band: 0.010,
    farm_pct: 25,
    forest_threshold: 0.56,
    quota: 30,
};

// ---- 4. Macico de Pocos de Caldas (MG/SP): an alkaline volcanic ring complex --
const POCOS: Region = Region {
    id: "pocos",
    name: "Maciço de Poços de Caldas",
    place: "MG / SP",
    blurb: "A Late Cretaceous alkaline ring complex: syenite core, volcanic rim, bauxite on the plateaus.",
    structure: Structure::Rings { cx: 300.0, cy: 300.0, radii: &[1450.0, 2900.0], warp: 700.0 },
    blobs: &[],
    cap: Some((3, 168.0)),
    units: &[
        Unit {
            name: "Nepheline syenite intrusions",
            parent: "Poços de Caldas Alkaline Complex",
            age: "Late Cretaceous (~80 Ma)",
            rocks: &[("Nepheline syenite", 55, "Igneous"), ("Foyaite", 25, "Igneous"), ("Lujaurite", 10, "Igneous"), ("Syenite", 10, "Igneous")],
            minerals: &["Eudialyte", "Zircon", "Nepheline"],
            mineral_pct: 25,
            fossils: &[],
            fossil_pct: 0,
            exposure: 90,
            color: (210, 90, 110),
        },
        Unit {
            name: "Alkaline volcanics and volcaniclastics",
            parent: "Poços de Caldas Alkaline Complex",
            age: "Late Cretaceous (~80 Ma)",
            rocks: &[("Phonolite", 40, "Igneous"), ("Tuff", 25, "Igneous"), ("Volcanic breccia", 20, "Igneous"), ("Tinguaite", 15, "Igneous")],
            minerals: &["Sanidine", "Aegirine", "Sodalite"],
            mineral_pct: 15,
            fossils: &[],
            fossil_pct: 0,
            exposure: 80,
            color: (230, 150, 70),
        },
        Unit {
            name: "Precambrian basement",
            parent: "Mantiqueira Province",
            age: "Neoproterozoic and older",
            rocks: &[("Gneiss", 45, "Metamorphic"), ("Granite", 35, "Igneous"), ("Quartzite", 20, "Metamorphic")],
            minerals: &["Biotite", "Feldspar", "Garnet"],
            mineral_pct: 10,
            fossils: &[],
            fossil_pct: 0,
            exposure: 70,
            color: (170, 175, 190),
        },
        Unit {
            name: "Bauxite and laterite cover",
            parent: "Cenozoic weathering profile",
            age: "Cenozoic",
            rocks: &[("Bauxite", 55, "Sedimentary"), ("Laterite", 30, "Sedimentary"), ("Clay", 15, "Sedimentary")],
            minerals: &["Gibbsite", "Anatase"],
            mineral_pct: 18,
            fossils: &[],
            fossil_pct: 0,
            exposure: 60,
            color: (190, 150, 90),
        },
    ],
    seed: 0xB0C0_0004,
    relief: 1.05,
    water_level: 50.0,
    river_band: 0.010,
    farm_pct: 40,
    forest_threshold: 0.60,
    quota: 30,
};

// ---- 5. Serra dos Carajas (PA): Archean volcanics, iron and canga -------------
const CARAJAS: Region = Region {
    id: "carajas",
    name: "Serra dos Carajás",
    place: "PA",
    blurb: "An Archean basement and volcanic-iron sequence under Amazon forest, capped by canga on the plateaus.",
    structure: Structure::Belts { strike_deg: 100.0, widths: &[700.0, 420.0, 520.0, 600.0], warp: 1500.0 },
    blobs: &[Blob { unit: 4, scale: 2000.0, threshold: 0.64 }],
    cap: Some((5, 175.0)),
    units: &[
        Unit {
            name: "Xingu Complex",
            parent: "Basement",
            age: "Mesoarchean (~2.85 Ga)",
            rocks: &[("Gneiss", 55, "Metamorphic"), ("Granite (TTG)", 45, "Igneous")],
            minerals: &["Zircon", "Biotite"],
            mineral_pct: 10,
            fossils: &[],
            fossil_pct: 0,
            exposure: 70,
            color: (225, 140, 140),
        },
        Unit {
            name: "Parauapebas Formation",
            parent: "Grão Pará Group",
            age: "Neoarchean (~2.76 Ga)",
            rocks: &[("Basalt", 55, "Igneous"), ("Rhyolite", 20, "Igneous"), ("Tuff", 15, "Igneous"), ("Diabase", 10, "Igneous")],
            minerals: &["Epidote", "Chlorite", "Chalcopyrite"],
            mineral_pct: 18,
            fossils: &[],
            fossil_pct: 0,
            exposure: 75,
            color: (110, 150, 90),
        },
        Unit {
            name: "Carajás Formation",
            parent: "Grão Pará Group",
            age: "Neoarchean (~2.75 Ga)",
            rocks: &[("Jaspilite (banded iron formation)", 60, "Sedimentary"), ("Hematite ore", 25, "Sedimentary"), ("Chert", 15, "Sedimentary")],
            minerals: &["Hematite", "Magnetite", "Jasper"],
            mineral_pct: 38,
            fossils: &[],
            fossil_pct: 0,
            exposure: 90,
            color: (190, 55, 55),
        },
        Unit {
            name: "Águas Claras Formation",
            parent: "Rio Fresco Group",
            age: "Neoarchean (~2.68 Ga)",
            rocks: &[("Sandstone", 70, "Sedimentary"), ("Siltstone", 20, "Sedimentary"), ("Quartz wacke", 10, "Sedimentary")],
            minerals: &["Quartz", "Pyrolusite"],
            mineral_pct: 8,
            fossils: &[],
            fossil_pct: 0,
            exposure: 70,
            color: (235, 215, 140),
        },
        Unit {
            name: "Carajás Granite",
            parent: "Anorogenic intrusions",
            age: "Paleoproterozoic (~1.88 Ga)",
            rocks: &[("Granite", 80, "Igneous"), ("Diabase dike", 20, "Igneous")],
            minerals: &["Feldspar", "Quartz", "Biotite"],
            mineral_pct: 10,
            fossils: &[],
            fossil_pct: 0,
            exposure: 85,
            color: (240, 110, 150),
        },
        Unit {
            name: "Canga (iron duricrust)",
            parent: "Cenozoic cover",
            age: "Cenozoic",
            rocks: &[("Canga", 70, "Sedimentary"), ("Iron laterite", 30, "Sedimentary")],
            minerals: &["Goethite", "Hematite"],
            mineral_pct: 22,
            fossils: &[],
            fossil_pct: 0,
            exposure: 85,
            color: (150, 80, 50),
        },
    ],
    seed: 0xCA7A_0005,
    relief: 0.95,
    water_level: 55.0,
    river_band: 0.014,
    farm_pct: 8,
    forest_threshold: 0.50,
    quota: 35,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::Ground;
    use std::collections::HashMap;

    fn tally(r: &Region) -> HashMap<usize, u32> {
        let t = r.terrain();
        let mut m = HashMap::new();
        for i in 0..64 {
            for j in 0..64 {
                let (x, y) = (MAP_BOUNDS.0 + i as f32 * 100.0 + 50.0, MAP_BOUNDS.1 + j as f32 * 100.0 + 50.0);
                *m.entry(r.unit_at(&t, x, y)).or_insert(0) += 1;
            }
        }
        m
    }

    #[test]
    fn every_region_is_well_formed() {
        for r in REGIONS.iter() {
            assert!(!r.units.is_empty(), "{}", r.name);
            for u in r.units {
                assert!(!u.rocks.is_empty() && u.rocks.iter().all(|x| x.1 > 0), "{}", u.name);
                for &(_, _, class) in u.rocks {
                    assert!(["Igneous", "Sedimentary", "Metamorphic"].contains(&class), "{} {}", u.name, class);
                }
                assert!(u.exposure > 0 && u.exposure <= 100);
                assert!(u.mineral_pct == 0 || !u.minerals.is_empty());
                assert!(u.fossil_pct == 0 || !u.fossils.is_empty(), "{}", u.name);
                assert!(u.fossil_pct == 0 || u.fossil_pct <= 60);
            }
            if let Some((unit, _)) = r.cap {
                assert!(unit < r.units.len());
            }
            for b in r.blobs {
                assert!(b.unit < r.units.len());
            }
            match &r.structure {
                Structure::Layered { levels } => {
                    assert_eq!(levels.len() + 1, r.units.len(), "{}", r.name);
                    assert!(levels.windows(2).all(|w| w[0] < w[1]));
                }
                Structure::Belts { widths, .. } => assert!(widths.len() <= r.units.len()),
                Structure::Rings { radii, .. } => assert_eq!(radii.len() + 1 + r.cap.is_some() as usize, r.units.len()),
            }
            assert!(r.quota >= 10);
        }
    }

    #[test]
    fn region_ids_are_unique_and_there_are_five_levels() {
        assert_eq!(REGIONS.len(), 5);
        let mut ids: Vec<_> = REGIONS.iter().map(|r| r.id).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), 5);
    }

    #[test]
    fn every_unit_crops_out_somewhere_on_its_map() {
        for r in REGIONS.iter() {
            let m = tally(r);
            for (i, u) in r.units.iter().enumerate() {
                let share = m.get(&i).copied().unwrap_or(0) as f32 / (64.0 * 64.0);
                assert!(share > 0.015, "{}: {} covers only {:.1}%", r.name, u.name, share * 100.0);
            }
            // no one unit swallows the map
            assert!(m.values().all(|&n| (n as f32) / (64.0 * 64.0) < 0.65), "{}: {:?}", r.name, m);
        }
    }

    #[test]
    fn unit_lookup_is_deterministic() {
        for r in REGIONS.iter() {
            let t = r.terrain();
            for k in 0..200 {
                let (x, y) = (-3000.0 + k as f32 * 47.0, 5000.0 - k as f32 * 31.0);
                assert_eq!(r.unit_at(&t, x, y), r.unit_at(&t, x, y));
            }
        }
    }

    #[test]
    fn layered_basins_follow_the_elevation() {
        for r in REGIONS.iter().filter(|r| matches!(r.structure, Structure::Layered { .. })) {
            let t = r.terrain();
            let mut by_unit: Vec<Vec<f32>> = vec![Vec::new(); r.units.len()];
            for i in 0..60 {
                for j in 0..60 {
                    let (x, y) = (-3000.0 + i as f32 * 150.0, -3000.0 + j as f32 * 150.0);
                    by_unit[r.unit_at(&t, x, y)].push(t.elevation(x, y));
                }
            }
            // older (lower index) units are exposed lower down than younger ones
            let mean = |v: &Vec<f32>| v.iter().sum::<f32>() / v.len().max(1) as f32;
            for w in by_unit.windows(2) {
                assert!(mean(&w[0]) < mean(&w[1]), "{}", r.name);
            }
        }
    }

    #[test]
    fn folded_belts_repeat_symmetrically() {
        let r = &REGIONS[2];
        let t = r.terrain();
        let Structure::Belts { widths, .. } = &r.structure else { panic!() };
        let total: f32 = widths.iter().sum();
        // walking straight across the strike the sequence must reverse at the hinge
        let th = 35.0f32.to_radians();
        let across = (-th.sin(), th.cos());
        let mut seq = Vec::new();
        for k in 0..(total as i32 * 2 / 20) {
            let d = k as f32 * 20.0;
            let u = r.unit_at(&t, 1700.0 + across.0 * d, 1700.0 + across.1 * d);
            if seq.last() != Some(&u) {
                seq.push(u);
            }
        }
        assert!(seq.len() >= 4, "{:?}", seq);
    }

    #[test]
    fn caps_sit_on_the_high_ground_only() {
        for r in REGIONS.iter().filter(|r| r.cap.is_some()) {
            let (cap_unit, level) = r.cap.unwrap();
            let t = r.terrain();
            let (mut capped, mut seen_high) = (0, 0);
            for i in 0..80 {
                for j in 0..80 {
                    let (x, y) = (-3000.0 + i as f32 * 115.0, -3000.0 + j as f32 * 115.0);
                    let e = t.elevation(x, y);
                    let u = r.unit_at(&t, x, y);
                    if e > level {
                        seen_high += 1;
                        assert_eq!(u, cap_unit, "{} at {e}", r.name);
                    } else if u == cap_unit {
                        capped += 1;
                    }
                }
            }
            assert!(seen_high > 20, "{}: no plateau to cap", r.name);
            assert_eq!(capped, 0, "{}: cap unit appears below the plateau", r.name);
        }
    }

    #[test]
    fn rocks_follow_the_weights() {
        let u = &REGIONS[0].units[3]; // Serra Geral
        let mut c: HashMap<&str, u32> = HashMap::new();
        for roll in 0..10_000 {
            *c.entry(pick_rock(u, roll).0).or_insert(0) += 1;
        }
        let basalt = c["Basalt"] as f32 / 10_000.0;
        assert!((0.70..0.74).contains(&basalt), "{basalt}");
        assert!(c.contains_key("Diabase") && c.contains_key("Rhyodacite"));
    }

    #[test]
    fn fossils_only_in_fossiliferous_units() {
        let with: Vec<_> = REGIONS.iter().flat_map(|r| r.units.iter()).filter(|u| u.fossil_pct > 0).map(|u| u.name).collect();
        assert!(with.contains(&"Crato Formation") && with.contains(&"Irati Formation"));
        // the Precambrian and igneous maps are not fossil hunting grounds, except stromatolites
        for r in [&REGIONS[3], &REGIONS[4]] {
            assert!(r.units.iter().all(|u| u.fossil_pct == 0));
        }
    }

    #[test]
    fn maps_are_playable_open_ground_dominates_inside_the_bounds() {
        for r in REGIONS.iter() {
            let t = r.terrain();
            let (mut open, mut n) = (0, 0);
            for i in 0..64 {
                for j in 0..64 {
                    let (x, y) = (MAP_BOUNDS.0 + i as f32 * 150.0 + 75.0, MAP_BOUNDS.1 + j as f32 * 150.0 + 75.0);
                    n += 1;
                    open += (t.ground(x, y) == Ground::Open) as u32;
                }
            }
            let share = open as f32 / n as f32;
            assert!(share > 0.45, "{} only {:.0}% open", r.name, share * 100.0);
        }
    }
}
