/**
 * @file Ecs grammar for tree-sitter
 * @author Jan Kleinmann <jan.kleinmann@proton.me>
 * @license MIT
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

export default grammar({
  name: 'my_language',

  // Equivalent to r"\s*" and comments in your match block
  extras: $ => [
    /\s/,
    /\/\/[^\n\r]*/,
  ],

  word: $ => $.id,

  rules: {
    programm: $ => seq(
      repeat($.import),
      repeat($.statement)
    ),

    // --- Imports ---
    import: $ => seq(
      'import',
      choice(
        seq($.id, optional($.alias_rule)),
        seq('native', $.string, $.string, optional($.alias_rule))
      ),
      ';'
    ),

    alias_rule: $ => seq('as', $.id),

    // --- Statements ---
    statement: $ => choice(
      $.non_returnable,
      $.returnable_statement,
      $.if,
      $.while,
      $.for
    ),

    returnable_statement: $ => seq($.returnable, ';'),

    non_returnable: $ => choice(
      seq($._assignment_or_decl_or_entity, ';'),
      $.function_definition,
      $.system_definition,
      $.struct_definition,
      $.group_definition,
      seq('return', $.returnable_or_if, ';')
    ),

    _assignment_or_decl_or_entity: $ => choice(
      $.declaration,
      $.assignment,
      $.register_rule,
      $.create_entity
    ),

    // --- Expressions & Logic (Precedence) ---
    returnable: $ => $.logic,

    returnable_or_if: $ => choice($.returnable, $.if),

    logic: $ => choice(
      $.comparison,
      prec.left(1, seq($.logic, '&&', $.logic)),
      prec.left(2, seq($.logic, '||', $.logic))
    ),

    comparison: $ => choice(
      $.math,
      prec.left(1, seq($.comparison, '==', $.comparison)),
      prec.left(1, seq($.comparison, '!=', $.comparison)),
      prec.left(1, seq($.comparison, '<', $.comparison)),
      prec.left(1, seq($.comparison, '<=', $.comparison)),
      prec.left(1, seq($.comparison, '>', $.comparison)),
      prec.left(1, seq($.comparison, '>=', $.comparison))
    ),

    math: $ => choice(
      $.term,
      prec.right(1, seq('!', $.math)),
      prec.right(1, seq('-', $.math)),
      prec.left(2, seq($.math, '*', $.math)),
      prec.left(2, seq($.math, '/', $.math)),
      prec.left(2, seq($.math, '%', $.math)),
      prec.left(3, seq($.math, '+', $.math)),
      prec.left(3, seq($.math, '-', $.math))
    ),

    term: $ => choice(
      $.primitive,
      $.member_call,
      seq('(', $.returnable, ')'),
      $.list_create,
      $.map_create,
      $.option_create,
      $.result_create,
      seq('weak', $.member_call)
    ),

    // --- Member Access (a.b.c()) ---
    member_call: $ => seq(
      $.member_access_segment,
      repeat(seq('.', $.member_access_segment))
    ),

    member_access_segment: $ => seq(
      $.id,
      optional(choice(
        seq('(', commaSep($.returnable_or_if), ')'),
        seq('{', optional($.struct_assignment_list), '}')
      ))
    ),

    // --- Definitions ---
    function_definition: $ => seq(
      'fn', $.id, '(', commaSep($.type_param_rule), ')',
      optional($.return_type_rule),
      $.block
    ),

    system_definition: $ => seq(
      'system', $.id, '(', commaSep(seq($.id, ':', $.id)), ')',
      optional($.querying_with_term),
      $.block
    ),

    struct_definition: $ => seq(
      'struct', $.id, '{',
      commaSep(choice($.function_definition, $.type_param_rule)),
      '}'
    ),

    // --- Control Flow ---
    if: $ => seq(
      'if', '(', $.returnable, ')', $.block,
      repeat(seq('else', 'if', '(', $.returnable, ')', $.block)),
      optional(seq('else', $.block))
    ),

    while: $ => seq('while', '(', $.returnable, ')', $.block),

    for: $ => seq(
      'for', '(',
      choice(
        seq($.id, 'in', $.returnable),
        seq(
          optional(choice($.declaration, $.assignment)), ';',
          optional($.returnable), ';',
          optional($.assignment)
        )
      ),
      ')',
      $.block
    ),

    block: $ => seq(
      '{',
      repeat($.statement),
      optional($.returnable),
      '}'
    ),

    // --- Low Level Rules ---
    declaration: $ => choice(
      seq($.id, ':=', $.returnable_or_if),
      seq('let', $.type_param_rule, '=', $.returnable_or_if),
      seq('spawn', $.id)
    ),

    assignment: $ => seq(
      $.id,
      choice('=', '+=', '-=', '*=', '/=', '%='),
      $.returnable_or_if
    ),

    type_param_rule: $ => choice(
      seq($.id, $.return_type_rule),
      'self',
      seq('weak', 'self')
    ),

    return_type_rule: $ => seq(':', $.return_type),

    return_type: $ => prec.left(choice(
      field('base', sep1($.id, '.')),
      seq('[', $.return_type, ']'),
      seq('{', $.return_type, '->', $.return_type, '}'),
      seq('weak', $.return_type),
      prec(1, seq($.return_type, '?')),
      prec(1, seq($.return_type, '!', $.return_type))
    )),

    // --- Helpers for Data structures ---
    list_create: $ => seq('[', commaSep($.returnable), ']'),

    map_create: $ => seq('{', commaSep(seq($.returnable, '->', $.returnable)), '}'),

    option_create: $ => choice(
      'none',
      seq('some', '(', $.returnable, ')')
    ),

    result_create: $ => choice(
      seq('ok', '(', $.returnable, ')'),
      seq('err', '(', $.returnable, ')')
    ),

    struct_assignment_list: $ => seq(
      $.id, ':', $.returnable_or_if,
      repeat(seq(',', $.id, ':', $.returnable_or_if)),
      optional(',')
    ),

    // --- Group Definitions ---
    group_definition: $ => seq(
      'group',
      $.id,
      '{',
      commaSep($.group_system_rule),
      '}'
    ),

    group_system_rule: $ => choice(
      $.id,                                 // Single system
      seq($.id, '->', $.id)                 // Ordered pair
    ),

    // --- Registration Logic ---
    register_rule: $ => seq(
      'register',
      choice(
        $._arrow_chain,                     // Chain: a -> b -> c
        seq($.id, 'after', $.id),           // After: a after b
        seq($.id, 'before', $.id)           // Before: a before b
      )
    ),

    _arrow_chain: $ => seq($.id, repeat1(seq('->', $.id))),

    // --- Entity Creation ---
    create_entity: $ => seq(
      'create',
      'entity',
      $.id,
      optional(seq('with', commaSep($.returnable)))
    ),

    // --- System Querying (The 'querying' block) ---
    querying_with_term: $ => seq(
      'querying',
      commaSep($.system_querying)
    ),

    system_querying: $ => seq(
      $.id,
      'as',
      $.system_querying_type
    ),

    system_querying_type: $ => choice(
      seq(choice('List', 'Single'), 'with', '{', commaSep($.id), optional(seq('%', '{', $.query_condition, '}')), '}'),
      'World',
      seq('Resource', 'of', $.id),
      seq('EventReader', 'for', $.id),
      seq('EventWriter', 'for', $.id)
    ),

    query_condition: $ => choice(
      $.id,
      seq('(', $.query_condition, ')'),
      prec.left(1, seq('!', $.query_condition)),
      prec.left(2, seq($.query_condition, '&&', $.query_condition)),
      prec.left(3, seq($.query_condition, '||', $.query_condition))
    ),

    // --- Basic Tokens ---
    primitive: $ => choice($.int, $.float, $.string, $.bool),
    int: $ => /[0-9]+/,
    float: $ => /[0-9]+\.[0-9]+/,
    bool: $ => choice('true', 'false'),
    string: $ => /"(\\.|[^"\\])*"/,
    id: $ => /[a-zA-Z][a-zA-Z_0-9]*/,
  }
});

// --- Helper functions for LALRPOP-like macros ---
function commaSep(rule) {
  return optional(sep1(rule, ','));
}

function sep1(rule, separator) {
  return seq(rule, repeat(seq(separator, rule)));
}
