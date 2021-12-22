Changelog
==

# 0.18.0-beta

Release specialised in removing allocations
- The structure chain doesn't use hashset anymore and use bitvec instead
- optimised the push_wth_feedback method and the remove_chain method
- this version is in beta because bitvec is also in beta

# 0.17

- Uses now "linked list" for storing stones in chains. Doesn't need the "thread-safe" feature anymore because we don't
  use any Rc or Arc now. So now it's thread safe by default

- Performance improved for the play method on game. Others method didn't improve.
- reworked naming things
- updated README.md
- the main doc imported the README.md