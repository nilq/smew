# smew
A game-oriented, mission-critical programming langauge


## Example

```
# A player that moves around, hello
# Also it has a sprite, yes

player -> human:
  looks:
    sprite:
      "/res/sprites/player.png"

      scale-x: 10
      scale-y: 10

  when-awake:
    print "hello world"
    move-to 100, 200 - 100

  when-press-space:
    print "ouch holy fuck"
```