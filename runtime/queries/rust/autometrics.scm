((attribute_item)*
 .
 (attribute_item
   (attribute
     (identifier) @attr))
 .
 (attribute_item)*
 .
 (function_item
   name: (identifier) @func.name)
 (#eq? @attr "autometrics"))


((attribute_item)*
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
 (#eq? @type.impl @type.target))


((impl_item
   type: (type_identifier) @type.impl
   body: (declaration_list
           (function_item
             name: (identifier) @func.name)))

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

 (#eq? @attr "autometrics")
 (#eq? @type.impl @type.target))


((attribute_item)*
 .
 (attribute_item
   (attribute
     (identifier) @attr))
 .
 (attribute_item)*
 .
 (struct_item
   name: (type_identifier) @type.target)

 (#eq? @attr "autometrics"))


((attribute_item)*
 .
 (attribute_item
   (attribute
     (identifier) @attr))
 .
 (attribute_item)*
 .
 (impl_item
   type: (type_identifier) @type.impl
   body: (declaration_list
           (function_item
             name: (identifier) @func.name)))

 (#eq? @attr "autometrics"))

;; It is impossible to do arbitrary levels of nesting, so we just try 1 module deep
;; Ref: https://github.com/tree-sitter/tree-sitter/discussions/981
(
 (mod_item
  name: (identifier) @mod.name
  body: (declaration_list
         (attribute_item)*
         .
         (attribute_item
                  (attribute
                           (identifier) @attr))
         .
         (attribute_item)*
         .
         (function_item
                  name: (identifier) @func.name)))
 (#eq? @attr "autometrics"))


(
 (mod_item
  name: (identifier) @mod.name
  body: (declaration_list

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
                  name: (identifier) @func.name)))))

 (#eq? @attr "autometrics")
 (#eq? @type.impl @type.target))


(
 (mod_item
  name: (identifier) @mod.name
  body: (declaration_list
         (impl_item
          type: (type_identifier) @type.impl
          body: (declaration_list
                 (function_item
                  name: (identifier) @func.name)))
         (attribute_item)*
         .
         (attribute_item
          (attribute
           (identifier) @attr))
         .
         (attribute_item)*
         .
         (struct_item
          name: (type_identifier) @type.target)))

 (#eq? @attr "autometrics")
 (#eq? @type.impl @type.target))


(
 (mod_item
  name: (identifier) @mod.name
  body: (declaration_list
         (attribute_item)*
         .
         (attribute_item
          (attribute
           (identifier) @attr))
         .
         (attribute_item)*
         .
         (struct_item
          name: (type_identifier) @type.target)))

 (#eq? @attr "autometrics"))


(
 (mod_item
  name: (identifier) @mod.name
  body: (declaration_list

         (attribute_item)*
         .
         (attribute_item
          (attribute
           (identifier) @attr))
         .
         (attribute_item)*
         .
         (impl_item
          type: (type_identifier) @type.impl
          body: (declaration_list
                 (function_item
                  name: (identifier) @func.name)))))

 (#eq? @attr "autometrics"))
