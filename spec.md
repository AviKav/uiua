# Operators and Precedence

These are all the Uiua operators, ordered by increasing precedence

| Operator(s)         | Name(s)                  | Associativity |
| ------------------- | ------------------------ | ------------- |
| <\|                 | Backpipe                 | Right         |
| \|>                 | Pipe                     | Left          |
| <>, ><              | Self, Flip               | Left          |
| /, //, \\, \\\\     | Left/Right Leaf/Tree     | Left          |
| =, !=, <, <=, >, >= | Comparisons              | Left          |
| +, -                | Addition, Subtraction    | Left          |
| *, %                | Multiplication, Division | Left          |
| ...                 | Double Composition       | Left          |
| .                   | Composition              | Left          |