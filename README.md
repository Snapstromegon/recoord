# Recoord

Recoord is a coordinate handling library with the ability to parse and serialize different coordinate formats like
- dms (50°10'20"N 10°25'30"E) Feature: `format_dms`
- dd (15.7445,20.345346) Feature: `format_dd`
- geohash (ezs42) Feature: `format_geohash`

It's also able to optionally resolve adresses to locations using the [Nominatim Openstreetmap API](https://nominatim.openstreetmap.org/) (enable the feature "resolve_osm" for this).
