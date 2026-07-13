# Pharos demo GIF — regenerate with: brew install vhs && vhs demo.tape
# Run from this directory (needs demo-setup.sh + pharos-output.txt next to it).
# No network, no real npx: a shell function streams pharos-output.txt, which is
# sanitized real pharos output. When the CLI's output format changes, update
# pharos-output.txt to match and re-record.

Output ../pharos-demo.gif

Set Shell "bash"
Set FontSize 30
Set Width 1840
Set Height 1460
Set Theme "Catppuccin Mocha"
Set TypingSpeed 40ms
Set Padding 32

Hide
Type "source ./demo-setup.sh && clear"
Enter
Sleep 500ms
Show

Type "# Dependabot: qs@6.15.0 is vulnerable. But qs isn't in my package.json... who pulls it in?"
Enter
Sleep 1.5s

Type "npx pharos-cli qs@6.15.0 --recursive"
Sleep 500ms
Enter
Sleep 4s

Sleep 7s
