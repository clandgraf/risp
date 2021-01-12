
(def defmacro '(macro (name param-list &rest body)
  (list 'def name (list 'quote (concat (list 'macro param-list) body)))))

(defmacro defun (name param-list &rest body)
  (list 'def name (list 'quote (concat (list 'fn param-list) body))))

(defun second (lst)
  (rest (first lst)))

(defun is-unquote (expr)
  (= 'unquote (first expr)))

(defun is-empty (lst)
  (= (length lst) 0))

;; (defun map (fun lst)
;;   (if (is-empty lst) lst
;;       (cons (fun (first lst))
;;             (map fun (rest lst)))))

;; (defmacro quasiquote (expr)
;;   (if (is-list expr)
;;       (if (is-unquote)
;;           (second expr)
;;           (map 'quasiquote expr))
;;       expr)
