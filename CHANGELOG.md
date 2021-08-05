# 0.17

- Uses now "linked list" for storing stones in go strings. Doesn't need the "thread-safe" feature because we don't use
  any Rc or Arc now.

- Performance improved for the play method on game. Others method didn't improve.
- reworked naming things