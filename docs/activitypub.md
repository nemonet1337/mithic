# Mithic ActivityPub Federation

## Overview

Mithic implements ActivityPub protocol for federation with other servers.

## Actor (User) Object

```json
{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://w3id.org/security/v1"
  ],
  "id": "https://example.com/users/username",
  "type": "Person",
  "preferredUsername": "username",
  "name": "Display Name",
  "summary": "Bio text",
  "inbox": "https://example.com/users/username/inbox",
  "outbox": "https://example.com/users/username/outbox",
  "followers": "https://example.com/users/username/followers",
  "following": "https://example.com/users/username/following",
  "publicKey": {
    "id": "https://example.com/users/username#main-key",
    "owner": "https://example.com/users/username",
    "publicKeyPem": "-----BEGIN PUBLIC KEY-----\n...\n-----END PUBLIC KEY-----"
  }
}
```

## WebFinger

```
GET /.well-known/webfinger?resource=acct:username@example.com
```

Response:
```json
{
  "subject": "acct:username@example.com",
  "aliases": [
    "https://example.com/users/username"
  ],
  "links": [
    {
      "rel": "self",
      "type": "application/activity+json",
      "href": "https://example.com/users/username"
    }
  ]
}
```

## Supported Activities

### Incoming
- `Create` - New notes
- `Delete` - Delete notes
- `Follow` - Follow requests
- `Undo` - Undo actions (unfollow, unlike)
- `Accept` - Accept follow requests
- `Reject` - Reject follow requests
- `Announce` - Boosts/renotes
- `Like` - Favourites
- `Update` - Profile updates

### Outgoing
- Same as incoming

## HTTP Signatures

All federation requests must include HTTP Signatures (draft-cavage-http-signatures-08).

## Delivery

Activities are delivered to remote instances via POST to the inbox URL.

## Notes

- Delivery is asynchronous using the Dragonfly queue
- Failed deliveries are retried with exponential backoff
- Remote actors are cached locally
