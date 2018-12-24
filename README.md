# Smew

A simple, markup-like programming language for scripting and configuring.

## Syntax

### Example

The syntax is specifically targeted beginners and new-coming programmers.

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

### Documentation

#### Records

Records are collections of other records, assignments and operations.

```
foo:
  baz:
    print "hello world"
```

Innovatively, records can inherit data from other records.

```
frog -> foo:
  quack:
    baz!
```

#### Calls

```
print "hello world"
```

```
join "hello ", "my name is ", "bobby"
```

#### Assignments

Static constant declaration is valid, and is limited to scope-specific value maps.

```
a = 100

tree:
  height:
    a + 100
```