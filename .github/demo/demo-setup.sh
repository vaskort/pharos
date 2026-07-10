# Sourced by demo.tape in a Hidden block. Shadows npx with a function that
# streams the canned pharos output so the GIF never depends on the network.
npx() {
  sleep 0.8
  while IFS= read -r line; do
    printf '%b\n' "$line"
    sleep 0.04
  done < ./pharos-output.txt
}
