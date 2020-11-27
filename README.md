# Roll-rs
roll-rs is a dice roller that allows one to roll arbitrary dice and potentially combine them with arithmetic 

## Usage Modes
**Normal usage**
```
$ roll d8 + 2d4
d8    d4
d8 + 2d4 = 8
3     1
      4
```
**Short mode**
```
$ roll -s d8 + 2d4
d8 + 2d4 = [3] + [1, 4] = 8
```
**Advanced mode**    
_this allows using dice rolls to determine the sides and amount of another roll_
```
$ roll -a (d8)d(2d4)
 d8 d7 d4
(d8)d(2d4) = 5
 2  3  3
    2  4

```

## Notation
**Standard**  
Standard notation allows you to roll any sided die any number of times
```shell script
d     # roll a single 20 sided die
1d20  # equivalent
```
**Percentile**  
You can use `%` as a shorthand for 100 sides
```shell script
3d%   # roll a percentile die 3 times and add them together
3d100 # equivalent
```
**Keep**  
The keep modifier allows you to roll multiple dice but only keep the highest or lowest result(s)
```shell script
4d8kh2 # roll a d8 4 times and keep the highest 2 rolls
4d8k2  # equivalent to the above
4d8kl1 # roll a d10 4 times and keep the lowest roll
```
**Drop**  
The keep modifier allows you to roll multiple dice but drop the highest or lowest result(s)
(Opposite of Keep).
```shell script
4d8dl2 # roll a d8 4 times and drop the lowest 2 rolls
4d8d2  # equivalent to the above
4d8dh1 # roll a d8 4 times and drop the highest roll
```

## Maths
Roll-rs supports the following arithmetic operators

| Operator | Description |
| -------- | ----------- |
| +        | Plus |
| -        | Minus |
| *        | Multiplication |
| /        | Division |
| //       | Integer division |
| mod      | Modulo |
| **       | Exponentiation |

Roll-rs follows the normal order of operations and also allows the use of parenthesis to affect this.

