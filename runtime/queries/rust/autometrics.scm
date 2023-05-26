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

(
 (attribute_item)*
 .
 (attribute_item
   (attribute
     (identifier) @attr))
 .
 (attribute_item)*
 .
 (struct_item
   name: (type_identifier) @type.target)

 (impl_item
   type: (type_identifier) @type.impl
   body: (declaration_list
           (function_item
             name: (identifier) @func.name)))

 (#eq? @attr "autometrics")
 (#eq? @type.impl @type.target)
)
