TodoHandler
- AddItem: title: str, content: str
- ListItems
- DeleteItem: id: str
- MarkDone: id: str


Task 1
TodoStore (trait)
- AddItem: title: str, content: str
- ListItems
- DeleteItem: id: str
- MarkDone: id: str


CLI:

> todos add 'Hello world, I am walid'
> todos list
-> [ ] Hello world, I am walid (id: 1)

> todos complete 1
> todos list
-> [x] Hello world, I am walid (id: 1)


Storage: Sqlite