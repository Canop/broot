ddoc

# deploy directly on the server: going through ~/dev/www/dystroy would republish
# that machine's stale copy of every other project
chmod -R a+rX site
rsync -av site/ dys@dystroy.org:prod/www.dystroy.org/broot/
