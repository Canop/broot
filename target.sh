# extract the target from the output of 
target=$(rustc -vV | sed 's/^host: \(.*\)/\1/
    t
    d' | head -1)

echo "$target"
