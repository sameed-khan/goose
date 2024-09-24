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
    - Define viewport on screen to watch for scroll (absolute coordinates; draw bbox)
      - Bbox needs to be "sticky" -- bound to image or whatever exists at that point, so resistant
      to being moved
    - When there is no difference between last scroll and this scroll
    - Continue scrolling; watch for update and update difference
    - When update stops, then stop scrolling
    - **Consider**: What is this component's API? It is not a `GUIVerb` -- what do other things 
    need to get from it?
  - [ ] Seek Scroll: scrolling to make a certain element appear
- `Input` - finds a textbox and inserts some text
  - [x] Normal Input: exactly as above
  - [x] Submit Input: exactly as above *and then press \<Enter\>*
- `Hover` - Mouses over a zone and then waits for an *expected* state change in a specific search region
  - [ ] hover
- [ ]`Copy` - Copies a text area
  - Drag from one point to another within visible area
  - Scroll down through text frame to copy all text by keeping cursor at bottom edge of frame
- `Paste` - Pastes into another text area
- `Check` - read something on a region of the screen and then evaluate against some condition
  - Need to elaborate more here about the level of expressivity people can have for condition checking and evaluation
  - Here is also where AI LLM engine could enable more sophisticated analysis of text
  
### Semantics
These are structures / constructs that do things like `if` statements, `loop`, parse information, and
so on.
- [ ] `Table` - interface for repeated actions over a set of rows
  - User Input: bounding box of table area and on-screen line drawing to define first row and column
    - QoL: repeat rows every `y` units and columns every `x` units to get structure of table
  - Use OCR to extract text of table and read in information
  - Select elements based on position in table
  - Also specify elements to click based on raw points (does not work if scrolling is needed)
  - These are all specified as "objects" of a for verb or something like that

### GUI
**Implementation Checklist**
- [ ] Basic navbar and interface structure
- [ ] Click and drag interface for defining zones to capture text
- [ ] QoL: Mouse coordinates around cursor
- [ ] QoL: Ruler to measure pixel distances on-screen


### Other General Questions to Investigate
- Is there a *text-invariant* form of template matching? 
  - This would be something that can capture the *general structure* of a GUI element that contains
  text but is invariant to whatever text is actually contained inside the element.
  OCR could then be deployed to match against actual text content and then select a specific element.
  - You can create some kind of a semantic GUI structure by doing edge filtering with Canny and then grouping connected components.
  If one component is *contained* inside another component then it can be regarded as a child in the hierarchy. 
    - See [Screen Parsing](https://arxiv.org/abs/2109.08763) -- paper that uses `Faster-RCNN` to construct semantic hierarchies of mobile application GUIs

- Current state of table AI models for interpreting tables?
- How are different interpretation backends structured and integrated?
- Goose extensions API? how will that work?

### Implementation Timeline
**Deadline: 10/26**

**9/20 - 9/22**
- GUI

**9/23 - 9/29**
- GUI

**9/30 - 10/6**
- Finish remaining GUI verbs

**10/7 - 10/13**
- DSL implementation

**10/14 - 10/18**
- Generate sectra program
- Complete slides on RSNA presentation

**10/19 - 10/26**
- Testing and deployment