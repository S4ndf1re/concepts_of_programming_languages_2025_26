## Info
Die Grammatik ist in Ohm geschrieben. Um auszuprobieren, welche Programme Teil der Sprache sind, kann die folgende Grammatik in 
"https://ohmjs.org/editor/"
eingefügt werden.
## Grammar in Ohm
Must {
  Program = Statement* 

  Statement = Expression
  |Structdef
  |Fndef
  |Whilestatement
  |Branchstatement
  | Comment


  Structdef= struct ident l_curl (ident colon ident (comma ident colon ident)*)? r_curl
  Fndef = fn ident l_paren Parameters r_paren l_curl Block r_curl
  Parameters = ident colon Type (comma ident colon Type)*
  
  Whilestatement= while l_paren Expression r_paren l_curl Block r_curl
  Block = Statement*
  
  Branchstatement = Ifstatement Elseifstatement* Elsestatement
  Ifstatement =  if l_paren Expression r_paren l_curl Block r_curl 
  Elseifstatement = elseif  l_paren Expression r_paren l_curl Block r_curl 
  Elsestatement = else l_curl Block r_curl 
  
  Comment = slash slash alnum* slash slash
  
  Expression = Assignment semicolon
  Assignment = let ident (colon Type)? assign ( StructAssignment |Assignment) --assign
  | LogicOr 
  StructAssignment = ident with  l_curl (ident colon Primary (comma ident colon Primary)*)? r_curl (rest default)?
  LogicOr = LogicAnd (or LogicOr)?
  LogicAnd =Equality (and LogicAnd)?
  Equality = Comparison ((eq|neq) Equality)?
  Comparison = Term ((lt|gt|lte|gte) Comparison)?
  Term= Factor ((plus|minus) Term)?
  Factor=Not ((star|slash)Factor)?
  Not= not Not --not
  | Unary
  Unary = (plus | minus)? Call
  Call = Primary (Calladds)?
  Calladds = l_paren Arguments? r_paren --call
  |"." ident --member
  Primary = List | ident | number
  List = l_bracket Arguments? r_bracket
  Arguments= Expression (comma Expression)*
  Map = Type (arrow Map)?
  Type = (int| float| string| bool|ident) (l_bracket r_bracket)?

  assign = "="
  or = "|"
  and = "|"
  eq = "=="
  neq = "!="
  lt = "<"
  gt = ">"
  lte = "<="
  gte= ">="
  plus = "+"
  minus = "-"
  star = "*"
  slash = "/"
  not = "!"
  comma = ","
  colon = ":"
  l_paren = "("
  r_paren = ")"
  l_bracket = "["
  r_bracket = "]"
  l_curl = "{"
  r_curl = "}"
  arrow = "->"
  semicolon = ";"
  
  
  let = "let"
  fn = "fn"
  while = "while"
  if = "if"
  elseif = "else if"
  else = "else"
  return = "return"
  with = "with"
  rest = "rest"
  default = "default"
  
  int = "int"
  float = "float"
  string = "str"
  bool = "bool"
  struct = "struct"
  

  // An identifier
  ident = letter alnum*

  // A number
  number =
    digit* "." digit+  -- fract
  | digit+             -- whole
  
}