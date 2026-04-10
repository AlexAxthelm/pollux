# Track ID 
---
priority: very low
---

The general idea on this one is that I listen to a lot of music podcasts and it would be really nice to have some way to identify what song is playing, jump in between songs as if it were a normal music playlist — not part of a podcast which always sort of gets treated as like one monolithic block of an MP3 file. So I want to be moving back and forth within the MP3 file.

Ideally, the best case scenario is that the podcast authors are including the tracklist as part of the podcast chapters and/or that they are including timestamps and track IDs in the description — something coming from the authors. I need a way to automatically recognize those things and parse them and present them to the user through whatever UI comes out of this. That's the best case scenario.

Some podcasts don't do that. They'll just include maybe a tracklist without necessarily timestamp information, or they may just have nothing. So if there is a tracklist without timestamps, I would want to have some way to break that up. I don't entirely know what that looks like, but it seems like some sort of automatic analysis might help there.

In general, if there's an authoritative source (i.e. from the authors) that has
useful timestamp info, we should use that, and (if the user wants) fall back to
less official listings

This might tap into social stuff where there's a common database across users of the timestamps for tracklists in various episodes.

