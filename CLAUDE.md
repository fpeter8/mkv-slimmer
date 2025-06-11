- Try to use expect instead of unwrap and explain the broken assumption in the error message

## Stream Processing Implementation

- Stream removal functionality has been implemented using mkvmerge
- Smart optimization detects when no processing is needed and uses hardlinking/copying instead
- Proper default flag management ensures only one stream per type is marked as default
- Comprehensive error handling with helpful messages for common failure scenarios
