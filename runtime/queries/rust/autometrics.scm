(
 (attribute_item)*
 .
 (attribute_item
   (attribute
     (identifier) @attr))
 .
 (attribute_item)*
 .
 (function_item
   name: (identifier) @func.name)
 (#eq? @attr "autometrics")
)
