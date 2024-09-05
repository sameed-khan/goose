# Notes
Development notes tracking project development.
Not meant to be documentation or a productive reference record.
Could be potentially helpful to someone trying to understand how this was built.
Left here in version control since it might be useful later.

## Development Scopes
Different arms of the project:

### Verb
Different 'verbs' that correspond one-to-one to scripting actions in `Honk`.
They are defined as a unit of action *that changes UI state*.<br>

**Implementation Checklist**
- `Click` - clicks a GUI element
  - [x] Determines UI state change in the immediate zone *around* the cursor
  - [x] Option for user to provide custom `check_zone`
- `Scroll` - scrolls an interface
  - [ ] Iterative Scroll: scrolling to move through a list and repeat an action on each element; ends scroll after the list no longer moves forward
  - [ ] Seek Scroll: scrolling to make a certain element appear
- `Input` - finds a textbox and inserts some text
  - [ ] Normal Input: exactly as above
  - [ ] Submit Input: exactly as above *and then press \<Enter\>*
- `Hover` - Mouses over a zone and then waits for an *expected* state change in a specific search region
  - [ ] hover
- `Check` - read something on a region of the screen and then evaluate against some condition
  - Need to elaborate more here about the level of expressivity people can have for condition checking and evaluation
  - Here is also where AI LLM engine could enable more sophisticated analysis of text

### GUI
**Implementation Checklist**
- [ ] Click and drag interface for defining zones to capture text
- [ ] QoL: Mouse coordinates around cursor
- [ ] Transparent overlay over screen
- [ ] QoL: Ruler to measure pixel distances on-screen


### Other General Questions to Investigate
- Is there a *text-invariant* form of template matching? 
  - This would be something that can capture the *general structure* of a GUI element that contains
  text but is invariant to whatever text is actually contained inside the element.
  OCR could then be deployed to match against actual text content and then select a specific element.

- Current state of table AI models for interpreting tables?
- How are different interpretation backends structured and integrated?
- Goose extensions API? how will that work?