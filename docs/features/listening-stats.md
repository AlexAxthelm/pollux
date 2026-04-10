# Listening Statistics
---
priority: mid
---


I'd like to be able to compile listening statistics about the podcasts that I've listened to.

Basically, I would like to break down the amount of time I've listened to each episode of a podcast, both in episode time and in wall time if I'm listening at like a 2x speed or something. And then I will want to be able to identify the amount of time saved from the various speed multipliers, silencing, etc.

But what I also want is to track time listening per device. Overall, a big thing that I'm thinking this is going to need is: basically, as we're listening to everything, we're going to need to run some sort of continuous log where it's episode listened, how much listened to, what device we're on, blah blah blah — all the important metadata about the actual listening events. And then we'll have to compile those across devices somehow (see sync), and once we have that we can get the grand totals of everything.

This is making me think that we can probably also use this to do, as part of sync, what the current head position is on a podcast — by figuring out which device listened to it most recently and what is the timestamp on that device.

In the future, we can extend this to do some automated "year in review" fun
stuff

I'd like for this to also help me identify listening time (and habits)
per-device (I listen to these podcasts on my phone, and these other ones on my
laptop)
