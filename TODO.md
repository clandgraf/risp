
Special Forms
--------------

def -- introduce/modify global variable
set -- introduce/modify variable in current scope
env -- create new scope

progn -- create a body
fn~ -- create lambda with single sexpr instead of body
if~ -- if expr without body

Basic Macros
-------------

let -> (env
  (progn
     (set a (fn1))
     (set b (fn2)
     ...
     @body)))

if @rest body ->
