# OdinSource

A tool for organizing and retreiving research reference documents.

## _Purpose_

When I was doing research for my masters thesis, I continually wished that I had
a tool for organizing reference documents.  I wanted to be able to give each
document a label (or multiple labels) which would allow me to retreive the
documents relavent to the aspect of research I was working on at the time.
This would have been especially handy for citations.

## _Goals_

1. Remove the need for clever and lengthy file naming
2. Provide a system for associating tags with a document
3. Provide a means to query a document library by tag(s)
4. Assist in preventing excessive bespoke tag values
5. Prevent storage of duplicate reference documents

## _TODO_

- [x] Add a complete document record via CLI options
- [x] Batch add document records via TOML file
- [x] Modify tag records
- [x] Modify document records (individual or multiple fields)
- [ ] Open document by ID or title
    - [x] Using `xdg-open`
- [ ] Query document records by tag(s)
- [ ] Configuration option for changing which program to use for opening documents
- [ ] Dialogs for preventing inadvertant accumulation of tags
- [ ] Export document records as bibtex (or other citation formats)
- [ ] Graphical user interface
- [ ] Cloud service and storage
- [ ] Parse PDF documents for metadata
- [ ] Expand file types beyond PDF
