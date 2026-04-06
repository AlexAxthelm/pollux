# Feed Subscription & Library

* Support RSS/Atom/JSON Feed
* Allow user to subscribe
* Present all feeds/subscriptions in library (all subscriptions are feeds, but
  also playlists are feeds; can filter which in library view)


## UI 

### Library view

* list view of subscriptions (may be grid view of feed art)
* Can refresh all feeds from here


Normal podcast URL (RSS/Atom/maybe JSON feed), parse it, store feed info to database, kickstart subscription machinery. Settings like how often to check, what to download. Title, artwork, description, normal metadata. Given URL, parse as RSS, confirm user wants to subscribe, store to database.

Need to manage multiple podcasts — no reason to limit to one. Library management. Subscribe, unsubscribe. See full list of episodes including ones that rotated out of feed.
