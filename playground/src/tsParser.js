import Parser from 'web-tree-sitter';

// Language queries - exact copies from Rust source
const QUERIES = {
  rust: {
    interface: `
;; Top-level function (source_file > function_item)
(source_file
  (function_item
    (visibility_modifier)? @vis
    name: (identifier) @name
    body: (block) @body) @item)

;; Struct
(source_file
  (struct_item
    (visibility_modifier)? @vis
    name: (type_identifier) @name) @item)

;; Enum
(source_file
  (enum_item
    (visibility_modifier)? @vis
    name: (type_identifier) @name) @item)

;; Trait
(source_file
  (trait_item
    (visibility_modifier)? @vis
    name: (type_identifier) @name) @item)

;; Impl block (methods extracted from node, not query)
(source_file
  (impl_item) @item)

;; Module
(source_file
  (mod_item
    (visibility_modifier)? @vis
    name: (identifier) @name) @item)

;; Use declaration
(source_file
  (use_declaration
    (visibility_modifier)? @vis) @item)

;; Const
(source_file
  (const_item
    (visibility_modifier)? @vis
    name: (identifier) @name) @item)

;; Static
(source_file
  (static_item
    (visibility_modifier)? @vis
    name: (identifier) @name) @item)

;; Type alias
(source_file
  (type_item
    (visibility_modifier)? @vis
    name: (type_identifier) @name) @item)

;; Macro definition
(source_file
  (macro_definition
    name: (identifier) @name) @item)
`,
    expand: `
(function_item
  name: (identifier) @name) @item

(struct_item
  name: (type_identifier) @name) @item

(enum_item
  name: (type_identifier) @name) @item

(trait_item
  name: (type_identifier) @name) @item

(impl_item
  type: (_) @impl_type) @item

(mod_item
  name: (identifier) @name) @item

(const_item
  name: (identifier) @name) @item

(static_item
  name: (identifier) @name) @item

(type_item
  name: (type_identifier) @name) @item

(macro_definition
  name: (identifier) @name) @item
`
  },
  typescript: {
    interface: `
; Top-level function declarations
(program
  (function_declaration
    name: (identifier) @name
    body: (statement_block) @body) @item)

; Exported function declarations
(program
  (export_statement
    (function_declaration
      name: (identifier) @name
      body: (statement_block) @body)) @item)

; Top-level class declarations
(program
  (class_declaration
    name: (type_identifier) @name
    body: (class_body) @body) @item)

; Exported class declarations
(program
  (export_statement
    (class_declaration
      name: (type_identifier) @name
      body: (class_body) @body)) @item)

; Top-level abstract class declarations
(program
  (abstract_class_declaration
    name: (type_identifier) @name
    body: (class_body) @body) @item)

; Exported abstract class declarations
(program
  (export_statement
    (abstract_class_declaration
      name: (type_identifier) @name
      body: (class_body) @body)) @item)

; Top-level interface declarations
(program
  (interface_declaration
    name: (type_identifier) @name
    body: (interface_body) @body) @item)

; Exported interface declarations
(program
  (export_statement
    (interface_declaration
      name: (type_identifier) @name
      body: (interface_body) @body)) @item)

; Top-level type alias declarations
(program
  (type_alias_declaration
    name: (type_identifier) @name) @item)

; Exported type alias declarations
(program
  (export_statement
    (type_alias_declaration
      name: (type_identifier) @name)) @item)

; Top-level enum declarations
(program
  (enum_declaration
    name: (identifier) @name
    body: (enum_body) @body) @item)

; Exported enum declarations
(program
  (export_statement
    (enum_declaration
      name: (identifier) @name
      body: (enum_body) @body)) @item)

; Top-level import statements
(program
  (import_statement) @item)

; Top-level lexical declarations (const/let)
(program
  (lexical_declaration
    (variable_declarator
      name: (identifier) @name)) @item)

; Exported lexical declarations
(program
  (export_statement
    (lexical_declaration
      (variable_declarator
        name: (identifier) @name))) @item)
`,
    expand: `
(function_declaration
  name: (identifier) @name
  body: (statement_block) @body) @item

(class_declaration
  name: (type_identifier) @name
  body: (class_body) @body) @item

(abstract_class_declaration
  name: (type_identifier) @name
  body: (class_body) @body) @item

(interface_declaration
  name: (type_identifier) @name
  body: (interface_body) @body) @item

(type_alias_declaration
  name: (type_identifier) @name) @item

(enum_declaration
  name: (identifier) @name
  body: (enum_body) @body) @item

(lexical_declaration
  (variable_declarator
    name: (identifier) @name)) @item

(import_statement) @item

(export_statement
  (function_declaration
    name: (identifier) @name
    body: (statement_block) @body)) @item

(export_statement
  (class_declaration
    name: (type_identifier) @name
    body: (class_body) @body)) @item

(export_statement
  (abstract_class_declaration
    name: (type_identifier) @name
    body: (class_body) @body)) @item

(export_statement
  (interface_declaration
    name: (type_identifier) @name
    body: (interface_body) @body)) @item

(export_statement
  (type_alias_declaration
    name: (type_identifier) @name)) @item

(export_statement
  (enum_declaration
    name: (identifier) @name
    body: (enum_body) @body)) @item

(export_statement
  (lexical_declaration
    (variable_declarator
      name: (identifier) @name))) @item
`
  },
  python: {
    interface: `
; Top-level function definitions
(module
  (function_definition
    name: (identifier) @name
    body: (block) @body) @item)

; Top-level decorated definitions (functions/classes with decorators)
(module
  (decorated_definition
    (function_definition
      name: (identifier) @name
      body: (block) @body)) @item)

; Top-level class definitions
(module
  (class_definition
    name: (identifier) @name
    body: (block) @body) @item)

; Top-level decorated class definitions
(module
  (decorated_definition
    (class_definition
      name: (identifier) @name
      body: (block) @body)) @item)

; Top-level import statements
(module
  (import_statement) @item)

; Top-level import-from statements
(module
  (import_from_statement) @item)

; Top-level assignments (constants)
(module
  (expression_statement
    (assignment
      left: (identifier) @name)) @item)
`,
    expand: `
(function_definition
  name: (identifier) @name
  body: (block) @body) @item

(decorated_definition
  (function_definition
    name: (identifier) @name
    body: (block) @body)) @item

(class_definition
  name: (identifier) @name
  body: (block) @body) @item

(decorated_definition
  (class_definition
    name: (identifier) @name
    body: (block) @body)) @item

(import_statement) @item

(import_from_statement) @item

(expression_statement
  (assignment
    left: (identifier) @name)) @item
`
  },
  javascript: {
    interface: `
; Top-level function declarations
(program
  (function_declaration
    name: (identifier) @name
    body: (statement_block) @body) @item)

; Exported function declarations
(program
  (export_statement
    (function_declaration
      name: (identifier) @name
      body: (statement_block) @body)) @item)

; Top-level class declarations
(program
  (class_declaration
    name: (identifier) @name
    body: (class_body) @body) @item)

; Exported class declarations
(program
  (export_statement
    (class_declaration
      name: (identifier) @name
      body: (class_body) @body)) @item)

; Top-level import statements
(program
  (import_statement) @item)

; Top-level lexical declarations (const/let)
(program
  (lexical_declaration
    (variable_declarator
      name: (identifier) @name)) @item)

; Exported lexical declarations
(program
  (export_statement
    (lexical_declaration
      (variable_declarator
        name: (identifier) @name))) @item)

; Top-level variable declarations (var)
(program
  (variable_declaration
    (variable_declarator
      name: (identifier) @name)) @item)

; Exported variable declarations
(program
  (export_statement
    (variable_declaration
      (variable_declarator
        name: (identifier) @name))) @item)
`,
    expand: `
(function_declaration
  name: (identifier) @name
  body: (statement_block) @body) @item

(class_declaration
  name: (identifier) @name
  body: (class_body) @body) @item

(lexical_declaration
  (variable_declarator
    name: (identifier) @name)) @item

(variable_declaration
  (variable_declarator
    name: (identifier) @name)) @item

(import_statement) @item

(export_statement
  (function_declaration
    name: (identifier) @name
    body: (statement_block) @body)) @item

(export_statement
  (class_declaration
    name: (identifier) @name
    body: (class_body) @body)) @item

(export_statement
  (lexical_declaration
    (variable_declarator
      name: (identifier) @name))) @item

(export_statement
  (variable_declaration
    (variable_declarator
      name: (identifier) @name))) @item
`
  }
};

// Parser cache
let parserCache = {};
let languageCache = {};
let initialized = false;

// Initialize tree-sitter
async function init() {
  if (initialized) return;
  await Parser.init({
    locateFile(scriptName) {
      return `/codeview/${scriptName}`;
    }
  });
  initialized = true;
}

// Get or create parser for language
async function getParser(language) {
  if (!parserCache[language]) {
    const parser = new Parser();
    const wasmFile = language === 'typescript' ? 'tree-sitter-tsx.wasm' : `tree-sitter-${language}.wasm`;
    const lang = await Parser.Language.load(`/codeview/${wasmFile}`);
    parser.setLanguage(lang);
    parserCache[language] = parser;
    languageCache[language] = lang;
  }
  return { parser: parserCache[language], language: languageCache[language] };
}

// Item kinds mapping
const ITEM_KIND_MAP = {
  'function_item': 'function',
  'function_declaration': 'function',
  'struct_item': 'struct',
  'enum_item': 'enum',
  'enum_declaration': 'enum',
  'trait_item': 'trait',
  'impl_item': 'impl',
  'mod_item': 'mod',
  'use_declaration': 'use',
  'const_item': 'const',
  'static_item': 'static',
  'type_item': 'typealias',
  'type_alias_declaration': 'typealias',
  'macro_definition': 'macrodef',
  'class_declaration': 'class',
  'abstract_class_declaration': 'class',
  'class_definition': 'class',
  'interface_declaration': 'interface',
  'import_statement': 'use',
  'import_from_statement': 'use',
  'lexical_declaration': 'const',
  'variable_declaration': 'const',
  'expression_statement': 'const',
  'decorated_definition': null, // Will be determined by child
  'export_statement': null, // Will be determined by child
  'method_definition': 'method',
};

function getItemKind(node) {
  if (node.type === 'decorated_definition' || node.type === 'export_statement') {
    // Look at the first named child
    for (let i = 0; i < node.namedChildCount; i++) {
      const child = node.namedChild(i);
      const kind = ITEM_KIND_MAP[child.type];
      if (kind) return kind;
    }
  }
  return ITEM_KIND_MAP[node.type] || 'unknown';
}

// Visibility parsing
function getVisibility(node, source) {
  // Look for visibility_modifier child
  for (let i = 0; i < node.childCount; i++) {
    const child = node.child(i);
    if (child.type === 'visibility_modifier') {
      const text = source.substring(child.startIndex, child.endIndex);
      if (text.includes('crate')) return 'crate';
      if (text.includes('super')) return 'super';
      if (text.startsWith('pub')) return 'public';
    }
  }
  // For TypeScript/JavaScript, check if inside export_statement
  let current = node.parent;
  while (current) {
    if (current.type === 'export_statement') return 'public';
    current = current.parent;
  }
  // Python: names starting with _ are private
  const name = getNodeName(node, source);
  if (name && name.startsWith('_') && !name.startsWith('__')) {
    return 'private';
  }
  return node.type === 'export_statement' ? 'public' : 'private';
}

// Extract name from node
function getNodeName(node, source) {
  const nameNode = node.childForFieldName('name');
  if (nameNode) {
    return source.substring(nameNode.startIndex, nameNode.endIndex);
  }
  // For impl blocks in Rust
  if (node.type === 'impl_item') {
    const typeNode = node.childForFieldName('type');
    if (typeNode) {
      return source.substring(typeNode.startIndex, typeNode.endIndex);
    }
  }
  return null;
}

// Find attribute start (for Rust items with #[...] attributes)
function findAttrStart(node) {
  let current = node;
  let prev = node.previousSibling;
  
  while (prev && prev.type === 'attribute_item') {
    current = prev;
    prev = prev.previousSibling;
  }
  
  return current.startIndex;
}

// Collapse body - replaces function body with { ... }
function collapseBody(source, itemStart, itemEnd, bodyStart, bodyEnd) {
  const before = source.substring(itemStart, bodyStart);
  const after = source.substring(bodyEnd, itemEnd);
  
  const beforeTrimmed = before.replace(/[\n\r]+$/, '');
  
  const collapsed = beforeTrimmed.endsWith(' ') || beforeTrimmed.endsWith('\t')
    ? `${beforeTrimmed}{ ... }${after.trim()}`
    : `${beforeTrimmed} { ... }${after.trim()}`;
  
  const startLine = source.substring(0, itemStart).split('\n').length;
  const mappings = buildSourceLineMappings(collapsed, startLine);
  
  return { collapsed, mappings };
}

// Collapse Python body - replaces with : ...
function collapsePythonBody(source, itemStart, itemEnd, bodyStart, bodyEnd) {
  const before = source.substring(itemStart, bodyStart);
  const after = source.substring(bodyEnd, itemEnd);
  
  // Find the colon before the body
  const colonIdx = before.lastIndexOf(':');
  const beforeColon = colonIdx >= 0 ? before.substring(0, colonIdx + 1) : before;
  
  const collapsed = `${beforeColon.trimEnd()} ...${after.trim()}`;
  
  const startLine = source.substring(0, itemStart).split('\n').length;
  const mappings = buildSourceLineMappings(collapsed, startLine);
  
  return { collapsed, mappings };
}

// Collapse impl/class block - keeps signatures, collapses method bodies
function collapseBlock(source, language, itemStart, blockNode) {
  const bodyRanges = [];
  collectFnBodies(blockNode, bodyRanges);
  bodyRanges.sort((a, b) => a[0] - b[0]);
  
  const endByte = blockNode.endIndex;
  let result = '';
  let pos = itemStart;
  
  for (const [bodyStart, bodyEnd] of bodyRanges) {
    result += source.substring(pos, bodyStart);
    if (language === 'python') {
      result += ': ...';
    } else {
      result += '{ ... }';
    }
    pos = bodyEnd;
  }
  result += source.substring(pos, endByte);
  
  const startLine = source.substring(0, itemStart).split('\n').length;
  const mappings = buildSourceLineMappings(result, startLine);
  
  return { collapsed: result, mappings };
}

// Collect function body ranges recursively
function collectFnBodies(node, ranges) {
  // Visit all children
  for (let i = 0; i < node.namedChildCount; i++) {
    const child = node.namedChild(i);
    
    // Check if this is a function/method
    if (['function_item', 'function_definition', 'method_definition', 
         'function_declaration'].includes(child.type)) {
      const bodyNode = child.childForFieldName('body');
      if (bodyNode) {
        ranges.push([bodyNode.startIndex, bodyNode.endIndex]);
      }
    }
    
    // Recurse for nested structures
    if (child.namedChildCount > 0) {
      collectFnBodies(child, ranges);
    }
  }
}

// Build source line mappings
function buildSourceLineMappings(content, startLine) {
  const lines = content.split('\n');
  return lines.map((line, i) => [startLine + i, line]);
}

// Extract methods from Rust impl block
function extractRustMethods(source, blockNode) {
  const methods = [];
  const declList = blockNode.childForFieldName('body');
  if (!declList || declList.type !== 'declaration_list') {
    return methods;
  }
  
  for (let i = 0; i < declList.namedChildCount; i++) {
    const child = declList.namedChild(i);
    if (child.type !== 'function_item') continue;
    
    const effectiveStart = findAttrStart(child);
    const lineStart = source.substring(0, effectiveStart).split('\n').length;
    const bodyNode = child.childForFieldName('body');
    
    if (bodyNode) {
      const { collapsed, mappings } = collapseBody(
        source,
        effectiveStart,
        child.endIndex,
        bodyNode.startIndex,
        bodyNode.endIndex
      );
      
      methods.push({
        kind: 'method',
        name: getNodeName(child, source),
        visibility: getVisibility(child, source),
        lineStart,
        lineEnd: child.endPosition.row + 1,
        content: collapsed,
        lineMappings: mappings,
        signature: buildRustFnSignature(source, child),
        body: '{ ... }'
      });
    }
  }
  
  return methods;
}

// Build Rust function signature
function buildRustFnSignature(source, node) {
  const parts = [];
  
  for (let i = 0; i < node.childCount; i++) {
    const child = node.child(i);
    if (['visibility_modifier', 'async', 'const', 'unsafe', 'extern'].includes(child.type)) {
      parts.push(source.substring(child.startIndex, child.endIndex));
    }
  }
  
  parts.push('fn');
  
  const nameNode = node.childForFieldName('name');
  if (nameNode) parts.push(source.substring(nameNode.startIndex, nameNode.endIndex));
  
  const tpNode = node.childForFieldName('type_parameters');
  if (tpNode) parts.push(source.substring(tpNode.startIndex, tpNode.endIndex));
  
  const paramsNode = node.childForFieldName('parameters');
  if (paramsNode) parts.push(source.substring(paramsNode.startIndex, paramsNode.endIndex));
  
  const retNode = node.childForFieldName('return_type');
  if (retNode) {
    parts.push('->');
    parts.push(source.substring(retNode.startIndex, retNode.endIndex));
  }
  
  for (let i = 0; i < node.childCount; i++) {
    const child = node.child(i);
    if (child.type === 'where_clause') {
      parts.push(source.substring(child.startIndex, child.endIndex));
    }
  }
  
  return parts.join(' ');
}

// Extract methods from TypeScript/JavaScript class
function extractTSMethods(source, classBody) {
  const methods = [];
  
  for (let i = 0; i < classBody.namedChildCount; i++) {
    const child = classBody.namedChild(i);
    if (child.type !== 'method_definition' && child.type !== 'public_field_definition') continue;
    
    const lineStart = child.startPosition.row + 1;
    const bodyNode = child.childForFieldName('body');
    
    if (bodyNode) {
      const { collapsed, mappings } = collapseBody(
        source,
        child.startIndex,
        child.endIndex,
        bodyNode.startIndex,
        bodyNode.endIndex
      );
      
      methods.push({
        kind: 'method',
        name: getNodeName(child, source),
        visibility: source.substring(child.startIndex, child.endIndex).includes('private') ? 'private' : 'public',
        lineStart,
        lineEnd: child.endPosition.row + 1,
        content: collapsed,
        lineMappings: mappings,
        body: '{ ... }'
      });
    }
  }
  
  return methods;
}

// Extract methods from Python class
function extractPythonMethods(source, classBody) {
  const methods = [];
  
  for (let i = 0; i < classBody.namedChildCount; i++) {
    const child = classBody.namedChild(i);
    if (child.type !== 'function_definition' && child.type !== 'decorated_definition') continue;
    
    const funcNode = child.type === 'decorated_definition' 
      ? child.namedChild(child.namedChildCount - 1)
      : child;
    
    if (!funcNode || funcNode.type !== 'function_definition') continue;
    
    const lineStart = child.startPosition.row + 1;
    const bodyNode = funcNode.childForFieldName('body');
    
    if (bodyNode) {
      const { collapsed, mappings } = collapsePythonBody(
        source,
        child.startIndex,
        child.endIndex,
        bodyNode.startIndex,
        bodyNode.endIndex
      );
      
      methods.push({
        kind: 'method',
        name: getNodeName(funcNode, source),
        visibility: 'public',
        lineStart,
        lineEnd: child.endPosition.row + 1,
        content: collapsed,
        lineMappings: mappings,
        body: ': ...'
      });
    }
  }
  
  return methods;
}

// Main extraction function - interface view
async function extractInterface(source, language) {
  await init();
  const { parser, language: lang } = await getParser(language);
  
  const tree = parser.parse(source);
  const query = lang.query(QUERIES[language].interface);
  const matches = query.matches(tree.rootNode);
  
  const items = [];
  const seenStarts = new Set();
  
  for (const match of matches) {
    const itemNode = match.captures.find(c => c.name === 'item')?.node;
    if (!itemNode) continue;
    
    // Avoid duplicates
    if (seenStarts.has(itemNode.startIndex)) continue;
    seenStarts.add(itemNode.startIndex);
    
    const effectiveStart = itemNode.type === 'function_item' || itemNode.type === 'struct_item'
      ? findAttrStart(itemNode)
      : itemNode.startIndex;
    
    const lineStart = source.substring(0, effectiveStart).split('\n').length;
    const lineEnd = itemNode.endPosition.row + 1;
    const kind = getItemKind(itemNode);
    const name = getNodeName(itemNode, source);
    const visibility = getVisibility(itemNode, source);
    
    // Handle impl/class blocks with collapsed method bodies
    if (['impl_item', 'class_declaration', 'abstract_class_declaration', 'class_definition'].includes(itemNode.type)) {
      const bodyNode = itemNode.childForFieldName('body');
      
      if (bodyNode) {
        const { collapsed, mappings } = collapseBlock(source, language, effectiveStart, itemNode);
        
        items.push({
          kind,
          name,
          visibility,
          lineStart,
          lineEnd,
          content: collapsed,
          lineMappings: mappings
        });
      }
    } else {
      // Regular items (functions, structs, etc.)
      const bodyNode = itemNode.childForFieldName('body');
      
      if (bodyNode && ['function_item', 'function_declaration', 'function_definition'].includes(itemNode.type)) {
        // Collapse function body
        const collapseFunc = language === 'python' ? collapsePythonBody : collapseBody;
        const { collapsed, mappings } = collapseFunc(
          source,
          effectiveStart,
          itemNode.endIndex,
          bodyNode.startIndex,
          bodyNode.endIndex
        );
        
        items.push({
          kind,
          name,
          visibility,
          lineStart,
          lineEnd,
          content: collapsed,
          lineMappings: mappings,
          signature: language === 'rust' ? buildRustFnSignature(source, itemNode) : null,
          body: language === 'python' ? ': ...' : '{ ... }'
        });
      } else {
        // No body to collapse (structs, enums, type aliases, imports, etc.)
        const content = source.substring(effectiveStart, itemNode.endIndex);
        const mappings = buildSourceLineMappings(content, lineStart);
        
        items.push({
          kind,
          name,
          visibility,
          lineStart,
          lineEnd,
          content,
          lineMappings: mappings
        });
      }
    }
  }
  
  // Sort by line start
  items.sort((a, b) => a.lineStart - b.lineStart);
  
  return items;
}

// Main extraction function - expand view
async function extractExpand(source, language, symbols) {
  await init();
  const { parser, language: lang } = await getParser(language);
  
  const tree = parser.parse(source);
  const query = lang.query(QUERIES[language].expand);
  const matches = query.matches(tree.rootNode);
  
  const items = [];
  const seenStarts = new Set();
  const symbolSet = new Set(symbols.map(s => s.trim().toLowerCase()));
  
  for (const match of matches) {
    const itemNode = match.captures.find(c => c.name === 'item')?.node;
    if (!itemNode) continue;
    
    // Avoid duplicates
    if (seenStarts.has(itemNode.startIndex)) continue;
    
    const name = getNodeName(itemNode, source);
    
    // Filter by symbol names
    if (symbolSet.size > 0 && (!name || !symbolSet.has(name.toLowerCase()))) {
      continue;
    }
    
    seenStarts.add(itemNode.startIndex);
    
    const effectiveStart = itemNode.type === 'function_item' || itemNode.type === 'struct_item'
      ? findAttrStart(itemNode)
      : itemNode.startIndex;
    
    const lineStart = source.substring(0, effectiveStart).split('\n').length;
    const lineEnd = itemNode.endPosition.row + 1;
    const kind = getItemKind(itemNode);
    const visibility = getVisibility(itemNode, source);
    const content = source.substring(effectiveStart, itemNode.endIndex);
    
    items.push({
      kind,
      name,
      visibility,
      lineStart,
      lineEnd,
      content,
      lineMappings: null // Expand mode uses sequential lines
    });
  }
  
  // Sort by line start
  items.sort((a, b) => a.lineStart - b.lineStart);
  
  return items;
}

// Apply filters
function applyFilters(items, options) {
  const { pubOnly, fnsOnly, typesOnly, noTests } = options;
  const hasKindFilter = fnsOnly || typesOnly;
  
  return items.filter(item => {
    // --no-tests: skip mod tests
    if (noTests && item.kind === 'mod' && item.name === 'tests') {
      return false;
    }
    
    // --pub: public only
    if (pubOnly && item.visibility !== 'public') {
      return false;
    }
    
    // Kind filters
    if (hasKindFilter) {
      const isFn = item.kind === 'function' || item.kind === 'method';
      const isType = ['struct', 'enum', 'trait', 'typealias', 'class', 'interface'].includes(item.kind);
      
      let matched = false;
      if (fnsOnly && isFn) matched = true;
      if (typesOnly && isType) matched = true;
      if (!matched) return false;
      
      // When only --types, hide standalone methods
      if (item.kind === 'method' && !fnsOnly) {
        return false;
      }
    } else {
      // No kind filter: hide standalone methods (they're shown inside impl/class blocks)
      if (item.kind === 'method') {
        return false;
      }
    }
    
    return true;
  });
}

// Format output - plain text
function formatPlain(items, expandMode, fileName = 'input') {
  if (items.length === 0) return '';
  
  let output = '';
  
  if (expandMode) {
    // Expand mode: file::symbol [start:end] header for each item
    for (const item of items) {
      if (item.name) {
        output += `${fileName}::${item.name} [${item.lineStart}:${item.lineEnd}]\n`;
      } else {
        output += `${fileName} [${item.lineStart}:${item.lineEnd}]\n`;
      }
      output += formatItem(item) + '\n';
    }
  } else {
    // Interface mode: file header once, then items
    output += `${fileName}\n`;
    for (const item of items) {
      output += formatItem(item) + '\n';
    }
  }
  
  return output;
}

// Format a single item
function formatItem(item) {
  const maxLineNum = item.lineEnd;
  const width = maxLineNum.toString().length;
  let result = '';
  
  if (item.lineMappings) {
    // Use explicit line mappings (interface mode with collapsed bodies)
    for (const [lineNum, lineText] of item.lineMappings) {
      result += `${lineNum.toString().padStart(width)} | ${lineText}\n`;
    }
  } else {
    // Sequential line numbers (expand mode)
    const lines = item.content.split('\n');
    for (let i = 0; i < lines.length; i++) {
      const lineNum = item.lineStart + i;
      result += `${lineNum.toString().padStart(width)} | ${lines[i]}\n`;
    }
  }
  
  return result.trimEnd();
}

// Format output - JSON
function formatJson(items) {
  return JSON.stringify({
    files: [{
      path: 'input',
      items: items.map(item => ({
        kind: item.kind,
        name: item.name,
        visibility: item.visibility,
        line_start: item.lineStart,
        line_end: item.lineEnd,
        signature: item.signature,
        body: item.body,
        content: item.content
      }))
    }]
  }, null, 2);
}

// Format output - stats
function formatStats(items, source) {
  const lines = source.split('\n').length;
  const bytes = source.length;
  
  const kindCounts = {};
  for (const item of items) {
    kindCounts[item.kind] = (kindCounts[item.kind] || 0) + 1;
  }
  
  let output = 'Stats\n\n';
  output += `Files: 1\n`;
  output += `Lines: ${lines}\n`;
  output += `Bytes: ${bytes}\n`;
  output += `Items: ${items.length}\n\n`;
  
  if (Object.keys(kindCounts).length > 0) {
    output += 'By kind:\n';
    for (const [kind, count] of Object.entries(kindCounts).sort()) {
      output += `  ${kind}: ${count}\n`;
    }
  }
  
  return output;
}

// Main parse function
export async function parseCode(source, language, options) {
  try {
    const expandMode = options.symbols && options.symbols.length > 0;
    const symbols = expandMode ? options.symbols.split(',').map(s => s.trim()).filter(Boolean) : [];
    
    let items;
    if (expandMode) {
      items = await extractExpand(source, language, symbols);
    } else {
      items = await extractInterface(source, language);
    }
    
    // Apply filters
    items = applyFilters(items, options);
    
    // Format output
    if (options.stats) {
      return formatStats(items, source);
    } else if (options.json) {
      return formatJson(items);
    } else {
      return formatPlain(items, expandMode);
    }
  } catch (error) {
    console.error('Parse error:', error);
    return `Error: ${error.message}\n${error.stack}`;
  }
}
