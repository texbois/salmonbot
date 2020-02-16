# salmonbot

a vkontakte bot implementing some neat text&image-based quest challenges

## prerequisites

* a recent version of rust (developed on 1.41)
* an instance of redis accessible on `redis://127.0.0.1/`

## community setup

1. go to *settings* -> *api usage*
2. create a new token with the following rights: *community management*, *community messages*, *photos*
3. switch to the *long poll api* tab
4. enable it
5. uncheck all event types but *message received*

## static data

`mkdir static` and put all static content (texts and images) in there.
if you are not sure what files you need, move along to the next step,
the compiler will error out on missing entries

## getting up and running

```
cargo run -- <behavior>
```

see below for a list of available behaviors

## behaviors

#### chest

perform perceptual image comparison against a hardcoded hash,
additionally preventing the player from participating more than once
(the player's id is stored/looked up in a set)

#### stone

control the player's progression through the challenge by placing
their id in buckets (sets) according to the submitted image

the challenge is split into several stages with distinct images.
to advance to the next one, the player's id needs to exist
in all buckets for the current stage

#### test

reply with perceptual hashes of submitted images —
handy for development and testing
