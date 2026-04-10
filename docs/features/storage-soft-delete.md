# Soft Unsubscribe
---
priority: mid
---

Unsubscribing should be a "soft" action (mark feed as
Unsubscribed/inactive/whatever), and then after a timeout (30 days) remove from
db (actual delete of metadata). when the user unsubscribes, there should be a
note that says "we'll hold on to some details about this for (30 days) if you
want to resubscribe" so that things like play history and such are preserved for
a bit, but there should also be a button there that says "delete now" which does
that.

If the user re-subscribes in the grace window, then the general behavior should
be that the un-subscribe didn't happen (restore played status on episodes, etc)


