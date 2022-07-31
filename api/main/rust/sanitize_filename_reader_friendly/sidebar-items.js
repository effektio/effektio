window.SIDEBAR_ITEMS = {"constant":[["ALWAYS_REPLACED_OR_FILTERED_CHARS","A set of characters that is always replaced or filtered and will never appear in the output stream. Please note that additionally to the above : all `is_whitespace()` characters are always replaced by space and all `is_control()` characters are always filtered."],["POTENTIALLY_REPLACED_CHARS","An unordered list of all characters that are potentially replaced under certain conditions. Please note that additionally to the above : all `is_whitespace()` characters are always replaced by space and all `is_control()` characters are always filtered."],["TRIM_LINE_CHARS","Group characters into lines (separated by newlines) and trim both sides of all lines by the set of the quoted characters. In addition to the listed characters whitespace is trimmed too. As the filter operates line by line, it guarantees, that none of the listed characters can appear at the beginning or the at end of the output string."]],"fn":[["sanitize","Converts strings in a file system friendly and human readable form."]]};