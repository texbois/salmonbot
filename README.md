# salmonbot

a vkontakte bot implementing some neat text&image-based quest challenges

## prerequisites

* a recent version of rust (developed on 1.41)
* an instance of redis accessible on `redis://127.0.0.1/`

## getting up and running

```
cargo run -- <behavior>
```

see below for a list of available behaviors

## behaviors

#### chest

the bot performs perceptual image comparison against a hardcoded hash,
additionally checking whether has already participated (boolean state per user ID)

#### test

the bot replies with perceptual hashes of images it receives —
handy for development and testing
