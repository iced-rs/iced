viewport: 500x800
mode: Impatient
preset: Empty
-----
click at "What needs to be done?"
type "Create the universe"
type enter
type "Make an apple pie"
type enter
expect "2 tasks left"
click at "Create the universe"
expect "1 task left"
click at "Make an apple pie"
expect "0 tasks left"
