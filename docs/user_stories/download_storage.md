# User Stories: Download & storage

As a user
I want to have the episode I was listening to be immediately available to resume
so that I don't have to look for it.

As a user
I want to download episodes
so that I can listen offline

As a user
I want to limit how much space podcasts use,
so that I can have more space for my photos 

As a user
I want to bulk delete them
so that I can free up space conveniently

As a user
I want to see what shows are taking up space,
so that I can decide if I want to keep them

As a user
I want to see episodes coded by show
so that I can decide if I want to keep them

As a user
I want to see episodes sorted by size
so that I can decide if I want to keep them

As a user
I want to see episodes sorted by date
so that I can decide if I want to keep them

As a user
I want to have unlistened episodes delete themselves when a new one comes out
so that I don't have a bunch of episodes hanging around taking up space 

As a listener
I want to be able to have the newest episode of each show automatically
downloaded for me 
so that I don't need to do it myself

As a user
I want to set a space limit
so that I can keep space on device for other applications

As a subscriber
I want to remove all data related to a subscription
so that it's like I never subscribed

As a subscriber
I want to remove all episodes
so that space on my device is freed up

As a subscriber
I want to have new episodes be prepared automatically
so that I can listen without having to do anything

As a user
I want to know when I'm getting close to filling my storage limit
so that I can take applications

As a user
I want to know if a download failed
so that I can take action

As a subscriber
I want to know if a feed cannot be fetched
so that I can take action (re-subscribe/unsubscribe)

As a curator
I want to know if an episode cannot be downloaded
so that I can determine if I want to try again

As a user
I want to automatically re-try failed downloads in a limited way
so that transient errors don't bother me 

As a subscriber
I want to download episodes from access-controlled feeds
so that I can listen to them

As a curator
I want to see the files that are queued for download
so that I can know what will be a available on my device

As a curator
I want to control which devices a file gets downloaded to
so that I can manage space effectively

As a curator
I want to be able to cancel a download
so that it doesn't take space on my device

As a curator
I want to change the order of downloads
so that I can prioritize some files

As a curator
I want to know why a file is being downloaded
so that I can understand app behavior

As a curator
I want downloads to stop by default when storage reaches its limit
so that available files don't evict unexpectedly

As a user
I want to know if downloads are stopped because of storage limits
so that I know why new content isn't available

As a curator
I want to be able to set rules to evict or retain episodes based on storage
so that I don't have to manually manage storage

as a curator
I want to be able to set rules to evict or retain episodes on episode properties
(date, size, listened status)
so that I don't use storage on episodes I don't want to keep locally

As a curator
I want to set different retention rules for playlists
so that some playlists are always kept ready to listen

As a curator
I want to control how retention rules interact
so that conflicting retentions can be resolved in a way I expect
Note: related to settings, but these may inherit from multiple sources
(different playlists say to evict or retain)

As a curator,
I want conflicting retentions to act in a safe way by default (retain)
so that I don't accidentally lose files

As a curator
I want to be able to flag episodes as safe from eviction
so that they will always be available

As a curator
I want to have otherwise unavailable files be auto-marked as protected
so that I don't have to specifically protect against permanent data loss

As a curator
I want to have favorite/bookmarked files be auto-marked as protected
so that I don't have to specifically protect against data unavailability

As a User
I want files that are in the queue/active playlist to be downloaded (with high
priority)
so that they're available when my current episode finishes

As a user
I want to set a "storage style" from "full archive" type storage to "JIT" where
episodes are downloaded on the fly, with "the next few episodes available" in
between
so that I don't have to think about individual settings
Note: storage style is effectively a set of presets for storage-related settings

As a User
I want my user-level storage style to be available as playlist settings
so that I an set a different style for playlists

As a user
I want to be able to tweak individual storage settings while otherwise using the
storage style
so that I can retain fine control over storage

As a user
I want to the storage limit to be an upper limit, not reserved space
so that other apps on my device have space

As a user
I want to set storage limits as either space (GB) or time (hours)
so that I can use an intuitive unit when making decisions

As a user 
I want to see a conversion between time and space when setting limits
so that I can generally understand the consequences of my decisions

as a user
I want to see space limits as fraction of device whole
so that I can understand how it will impact my system overall
