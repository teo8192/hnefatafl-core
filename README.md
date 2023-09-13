# hnefatafl

Hnefatafl is a viking game, think of it as an old version of chess.

## Rules

The rules are quite simple: We have attackers and defenders on an 11x11 board set up like this:

-----------------------
| | | |A|A|A|A|A| | | |
-----------------------
| | | | | |A| | | | | |
-----------------------
| | | | | | | | | | | |
-----------------------
|A| | | | |D| | | | |A|
-----------------------
|A| | | |D|D|D| | | |A|
-----------------------
|A|A| |D|D|K|D|D| |A|A|
-----------------------
|A| | | |D|D|D| | | |A|
-----------------------
|A| | | | |D| | | | |A|
-----------------------
| | | | | | | | | | | |
-----------------------
| | | | | |A| | | | | |
-----------------------
| | | |A|A|A|A|A| | | |
-----------------------

There are three types of pieces:
 - Attackers
 - Defenders
 - King

For notation we can use a pair of numbers 12 to refer to a tile (1,2) that is zero indexed, and use X for the number 10.
In this way we have that 00 is the top left tile, 55 is the center, X0 is the bottom left and so on.

## Movement

The pieces move diagonally, like the rooks in chess.
They cannot "jump over" each other.

## Capturing a piece

An attacker or a defender may be captured if they have an opponent piece on both sides (the king can act to capture a piece), or if they have an opponent to one side and the corner tile (00, 0X, X0 and XX) to the other side.
They are only captured when the opponent moves into place to surround them, but are not captured if they themselves walk into said position.

The King may be captured if there are an attacker on all four adjacent tiles (i.e. the king cannot move anywhere).

### Example of defender capturing attacker

 1.
-------------
| | | | | | |
-------------
| |D|A| | |D|
-------------
| | | | | | |
-------------

 2.
-------------
| | | | | | |
-------------
| |D| |D| | |
-------------
| | | | | | |
-------------

### Example of attacker walking into being surrounded

 1.
-------------
| | | | | | |
-------------
| |D| |D| | |
-------------
| | |A| | | |
-------------

 2.
-------------
| | | | | | |
-------------
| |D|A|D| | |
-------------
| | | | | | |
-------------

## Winning

The defenders win when the king reaches one of the corner tiles (00, 0X, X0, XX), and the attackers win when they have captured the king.
