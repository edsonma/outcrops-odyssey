#!/usr/bin/env fish
# Packages the game as HTML5 with DragonRuby and copies it into ./web
# Run on your Mac (the bundled dragonruby binaries are macOS builds).
set root (realpath (dirname (status filename))/..)
cd $root/project; or exit 1

./dragonruby-publish --only-package mygame; or begin
    echo "dragonruby-publish failed (HTML5 export needs a licensed DragonRuby)"
    exit 1
end

set zip (ls -t builds/*html5*.zip 2>/dev/null | head -n 1)
if test -z "$zip"
    echo "No *html5*.zip found in project/builds"; exit 1
end

rm -rf $root/web; mkdir -p $root/web
unzip -q -o $zip -d $root/web; or exit 1
# some versions nest everything in one folder; flatten if so
if not test -f $root/web/index.html
    set inner (find $root/web -name index.html | head -n 1)
    if test -z "$inner"; echo "index.html not found in build"; exit 1; end
    set d (dirname $inner)
    mv $d/* $root/web/ 2>/dev/null
end
echo "Web build ready in $root/web — commit it and push to deploy."
