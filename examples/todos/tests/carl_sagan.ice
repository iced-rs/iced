viewport: 500x800
mode: Immediate
preset: Empty
-----
click "What needs to be done?"
type "Create the universe"
type enter
type "Make an apple pie"
type enter
expect "2 tasks left"
click "Create the universe"
expect "1 task left"
click "Make an apple pie"
expect "0 tasks left"
