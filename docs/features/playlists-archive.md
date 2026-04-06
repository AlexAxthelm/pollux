# Archive Binge Playlists

The core idea on this one is that there are a lot of podcasts that have very deep archives. Some podcasts are meant to be up-to-date — always listen to the latest episode, track the latest whatever. Other podcasts are evergreen, so for me these are like music podcasts, or Memory Palace, 99PI — those are like you can listen to any episode at any time, it's fine.

So what I would be looking for with this feature is the application helps me systematically work my way through all of the episodes. It would know about the entire backlog for the feed, and it would basically download the next episode on demand after I finished the last one. So there would always be an episode ready to go next, and it would have a log of all the ones that I have played and it would draw from the unplayed ones (default sequentially).

Ideally, this would interact with some playlist functionality where I could have a set of feeds, and it would switch between those feeds in the playlist so I wouldn't get a clump of like five episodes from the same feed all in a row because that part of the archive binge is way older than anything else on the playlist. That's something I run into right now where it's like I'll have three feeds and on one of them I'm working through episodes from like 2016 and on other ones I'm working through stuff in like 2018, so the stuff from the feed from 2016 is always at the beginning of the playlist.

Another nice feature would be if this could not interfere with tracking new episodes. In an example here: I listen to a podcast that is being currently released. I am also working through its archives. I would like to hear the new episode every week in addition to all of the other older episodes that I am listening to.

To avoid clumping, some strategies are: round-robin, proportional (more unplayed
episodes means more episodes in playlist), and date-normalized (treat all feed's
histories as unit length, and slot in episodes proportionally)

Functionally, this can exist as "playlist rules", but should have a good
interface, or be a suggested ruleset
