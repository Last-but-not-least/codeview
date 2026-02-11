export function parseCode(code, language, expanded) {
  const lines = code.split('\n')
  const items = extractItems(code, language)
  
  if (expanded && items.length > 0) {
    return formatExpandedView(items, lines)
  }
  
  return formatInterfaceView(lines, items)
}

function extractItems(code, language) {
  switch (language) {
    case 'rust':
      return extractRustItems(code)
    case 'typescript':
      return extractTypeScriptItems(code)
    case 'python':
      return extractPythonItems(code)
    case 'javascript':
      return extractJavaScriptItems(code)
    default:
      return []
  }
}

function extractRustItems(code) {
  const items = []
  const lines = code.split('\n')
  
  // Match structs
  const structRegex = /^(?:#\[.*\]\s*)*(?:pub\s+)?struct\s+(\w+)/gm
  let match
  while ((match = structRegex.exec(code)) !== null) {
    const startLine = code.substring(0, match.index).split('\n').length
    const endLine = findClosingBrace(lines, startLine - 1)
    items.push({
      type: 'struct',
      name: match[1],
      startLine,
      endLine,
      signature: extractSignature(lines, startLine - 1, endLine),
      isPublic: match[0].includes('pub')
    })
  }
  
  // Match impl blocks
  const implRegex = /^impl\s+(?:<[^>]+>\s+)?(\w+)/gm
  while ((match = implRegex.exec(code)) !== null) {
    const startLine = code.substring(0, match.index).split('\n').length
    const endLine = findClosingBrace(lines, startLine - 1)
    const methods = extractRustMethods(lines, startLine - 1, endLine)
    items.push({
      type: 'impl',
      name: match[1],
      startLine,
      endLine,
      signature: `impl ${match[1]}`,
      body: methods.join('\n'),
      isPublic: true
    })
  }
  
  // Match standalone functions
  const fnRegex = /^(?:pub\s+)?(?:async\s+)?fn\s+(\w+)/gm
  while ((match = fnRegex.exec(code)) !== null) {
    const startLine = code.substring(0, match.index).split('\n').length
    if (!isInsideBlock(lines, startLine - 1, 'impl')) {
      const endLine = findClosingBrace(lines, startLine - 1)
      items.push({
        type: 'function',
        name: match[1],
        startLine,
        endLine,
        signature: extractMethodSignature(lines, startLine - 1, endLine),
        isPublic: match[0].includes('pub')
      })
    }
  }
  
  return items
}

function extractRustMethods(lines, start, end) {
  const methods = []
  
  for (let i = start + 1; i < end; i++) {
    const line = lines[i].trim()
    const fnMatch = line.match(/^(?:pub\s+)?(?:async\s+)?fn\s+(\w+)/)
    if (fnMatch) {
      const methodEnd = findClosingBrace(lines, i)
      const sig = extractMethodSignature(lines, i, methodEnd)
      methods.push(`${i + 1} |     ${sig} { ... }`)
      i = methodEnd
    }
  }
  
  return methods
}

function extractTypeScriptItems(code) {
  const items = []
  const lines = code.split('\n')
  
  // Match interfaces
  const interfaceRegex = /^export\s+interface\s+(\w+)/gm
  let match
  while ((match = interfaceRegex.exec(code)) !== null) {
    const startLine = code.substring(0, match.index).split('\n').length
    const endLine = findClosingBrace(lines, startLine - 1)
    items.push({
      type: 'interface',
      name: match[1],
      startLine,
      endLine,
      signature: extractSignature(lines, startLine - 1, endLine),
      isPublic: true
    })
  }
  
  // Match type aliases
  const typeRegex = /^export\s+type\s+(\w+)/gm
  while ((match = typeRegex.exec(code)) !== null) {
    const startLine = code.substring(0, match.index).split('\n').length
    items.push({
      type: 'type',
      name: match[1],
      startLine,
      endLine: startLine,
      signature: lines[startLine - 1],
      isPublic: true
    })
  }
  
  // Match classes
  const classRegex = /^export\s+class\s+(\w+)/gm
  while ((match = classRegex.exec(code)) !== null) {
    const startLine = code.substring(0, match.index).split('\n').length
    const endLine = findClosingBrace(lines, startLine - 1)
    const methods = extractTypeScriptMethods(lines, startLine - 1, endLine)
    items.push({
      type: 'class',
      name: match[1],
      startLine,
      endLine,
      signature: lines[startLine - 1],
      body: methods.join('\n'),
      isPublic: true
    })
  }
  
  // Match functions
  const fnRegex = /^export\s+(?:async\s+)?function\s+(\w+)/gm
  while ((match = fnRegex.exec(code)) !== null) {
    const startLine = code.substring(0, match.index).split('\n').length
    const endLine = findClosingBrace(lines, startLine - 1)
    items.push({
      type: 'function',
      name: match[1],
      startLine,
      endLine,
      signature: extractMethodSignature(lines, startLine - 1, endLine),
      isPublic: true
    })
  }
  
  return items
}

function extractTypeScriptMethods(lines, start, end) {
  const methods = []
  
  for (let i = start + 1; i < end; i++) {
    const line = lines[i].trim()
    const methodMatch = line.match(/^(?:private\s+|public\s+)?(?:async\s+)?(\w+)\s*\(/)
    if (methodMatch && !line.includes('=') && !line.startsWith('//')) {
      const methodEnd = findClosingBrace(lines, i)
      const sig = extractMethodSignature(lines, i, methodEnd)
      const visibility = line.includes('private') ? 'private' : 'public'
      methods.push(`${i + 1} |     ${visibility} ${sig} { ... }`)
      i = methodEnd
    }
  }
  
  return methods
}

function extractPythonItems(code) {
  const items = []
  const lines = code.split('\n')
  
  // Match classes
  const classRegex = /^class\s+(\w+)/gm
  let match
  while ((match = classRegex.exec(code)) !== null) {
    const startLine = code.substring(0, match.index).split('\n').length
    const endLine = findPythonBlockEnd(lines, startLine - 1)
    const methods = extractPythonMethods(lines, startLine - 1, endLine)
    items.push({
      type: 'class',
      name: match[1],
      startLine,
      endLine,
      signature: lines[startLine - 1],
      body: methods.join('\n'),
      isPublic: !match[1].startsWith('_')
    })
  }
  
  // Match functions
  const fnRegex = /^(?:async\s+)?def\s+(\w+)/gm
  while ((match = fnRegex.exec(code)) !== null) {
    const startLine = code.substring(0, match.index).split('\n').length
    if (!isInsidePythonClass(lines, startLine - 1)) {
      const endLine = findPythonBlockEnd(lines, startLine - 1)
      items.push({
        type: 'function',
        name: match[1],
        startLine,
        endLine,
        signature: extractPythonSignature(lines, startLine - 1),
        isPublic: !match[1].startsWith('_')
      })
    }
  }
  
  return items
}

function extractPythonMethods(lines, start, end) {
  const methods = []
  
  for (let i = start + 1; i <= end; i++) {
    const line = lines[i]
    if (!line) continue
    const defMatch = line.match(/^\s+(?:async\s+)?def\s+(\w+)/)
    if (defMatch) {
      const methodEnd = findPythonBlockEnd(lines, i)
      const sig = extractPythonSignature(lines, i)
      methods.push(`${i + 1} |     ${sig}: ...`)
      i = methodEnd
    }
  }
  
  return methods
}

function extractJavaScriptItems(code) {
  const items = []
  const lines = code.split('\n')
  
  // Match classes
  const classRegex = /^export\s+class\s+(\w+)/gm
  let match
  while ((match = classRegex.exec(code)) !== null) {
    const startLine = code.substring(0, match.index).split('\n').length
    const endLine = findClosingBrace(lines, startLine - 1)
    const methods = extractJavaScriptMethods(lines, startLine - 1, endLine)
    items.push({
      type: 'class',
      name: match[1],
      startLine,
      endLine,
      signature: lines[startLine - 1],
      body: methods.join('\n'),
      isPublic: true
    })
  }
  
  // Match functions
  const fnRegex = /^export\s+(?:async\s+)?function\s+(\w+)/gm
  while ((match = fnRegex.exec(code)) !== null) {
    const startLine = code.substring(0, match.index).split('\n').length
    const endLine = findClosingBrace(lines, startLine - 1)
    items.push({
      type: 'function',
      name: match[1],
      startLine,
      endLine,
      signature: extractMethodSignature(lines, startLine - 1, endLine),
      isPublic: true
    })
  }
  
  // Match non-exported functions
  const privateFnRegex = /^(?:async\s+)?function\s+(\w+)/gm
  while ((match = privateFnRegex.exec(code)) !== null) {
    if (!code.substring(Math.max(0, match.index - 10), match.index).includes('export')) {
      const startLine = code.substring(0, match.index).split('\n').length
      const endLine = findClosingBrace(lines, startLine - 1)
      items.push({
        type: 'function',
        name: match[1],
        startLine,
        endLine,
        signature: extractMethodSignature(lines, startLine - 1, endLine),
        isPublic: false
      })
    }
  }
  
  return items
}

function extractJavaScriptMethods(lines, start, end) {
  const methods = []
  
  for (let i = start + 1; i < end; i++) {
    const line = lines[i].trim()
    const methodMatch = line.match(/^(?:async\s+)?(\w+)\s*\(/)
    if (methodMatch && !line.includes('=') && !line.startsWith('//')) {
      const methodEnd = findClosingBrace(lines, i)
      const sig = extractMethodSignature(lines, i, methodEnd)
      methods.push(`${i + 1} |     ${sig} { ... }`)
      i = methodEnd
    }
  }
  
  return methods
}

function formatInterfaceView(lines, items) {
  const output = []
  let lastLine = 0
  
  for (const item of items) {
    // Add imports/uses before first item
    if (lastLine === 0) {
      for (let i = 0; i < item.startLine - 1; i++) {
        const line = lines[i]
        if (line.trim() && (line.match(/^(use|import|from)/) || line.includes('@dataclass'))) {
          output.push(`${i + 1} | ${line}`)
        }
      }
      if (output.length > 0) output.push('')
    }
    
    if (item.type === 'impl' || item.type === 'class') {
      output.push(`${item.startLine} | ${item.signature} {`)
      if (item.body) {
        output.push(item.body)
      }
      output.push(`${item.endLine + 1} | }`)
    } else if (item.type === 'struct' || item.type === 'interface') {
      output.push(item.signature)
    } else if (item.type === 'function') {
      output.push(`${item.startLine} | ${item.signature} { ... }`)
    } else if (item.type === 'type') {
      output.push(`${item.startLine} | ${item.signature}`)
    }
    
    lastLine = item.endLine || item.startLine
    output.push('')
  }
  
  return output.join('\n')
}

function formatExpandedView(items, lines) {
  const output = []
  
  // Show first few interesting items in full
  const itemsToExpand = items.filter(item => 
    item.type === 'function' || item.type === 'class'
  ).slice(0, 2)
  
  for (const item of itemsToExpand) {
    output.push(`${item.name} [${item.startLine}:${item.endLine + 1}]`)
    for (let i = item.startLine - 1; i <= item.endLine; i++) {
      output.push(`${i + 1} | ${lines[i]}`)
    }
    output.push('')
  }
  
  return output.join('\n')
}

function findClosingBrace(lines, start) {
  let depth = 0
  let inString = false
  let stringChar = ''
  
  for (let i = start; i < lines.length; i++) {
    const line = lines[i]
    
    for (let j = 0; j < line.length; j++) {
      const char = line[j]
      const prevChar = j > 0 ? line[j - 1] : ''
      
      if ((char === '"' || char === "'" || char === '`') && prevChar !== '\\') {
        if (!inString) {
          inString = true
          stringChar = char
        } else if (char === stringChar) {
          inString = false
          stringChar = ''
        }
      }
      
      if (!inString) {
        if (char === '{') depth++
        if (char === '}') depth--
        
        if (depth === 0 && i > start) {
          return i
        }
      }
    }
  }
  
  return start
}

function findPythonBlockEnd(lines, start) {
  const baseIndent = lines[start].search(/\S/)
  
  for (let i = start + 1; i < lines.length; i++) {
    const line = lines[i]
    if (line.trim() === '') continue
    
    const indent = line.search(/\S/)
    if (indent <= baseIndent && indent !== -1) {
      return i - 1
    }
  }
  
  return lines.length - 1
}

function extractSignature(lines, start, end) {
  const result = []
  
  for (let i = start; i <= Math.min(end, start + 20); i++) {
    const line = lines[i]
    result.push(`${i + 1} | ${line}`)
    
    if (line.includes('{')) {
      // For structs/interfaces, show content until closing brace
      for (let j = i + 1; j <= end; j++) {
        result.push(`${j + 1} | ${lines[j]}`)
      }
      break
    }
  }
  
  return result.join('\n')
}

function extractMethodSignature(lines, start, end) {
  let sig = lines[start].trim()
  let i = start
  
  while (i < end && !sig.includes('{')) {
    i++
    if (i < lines.length) {
      sig += ' ' + lines[i].trim()
    }
  }
  
  return sig.replace(/\{.*$/, '').trim()
}

function extractPythonSignature(lines, start) {
  let sig = lines[start].trim()
  let i = start
  
  while (!sig.includes(':') && i < lines.length - 1) {
    i++
    sig += ' ' + lines[i].trim()
  }
  
  return sig.replace(/:.*$/, '')
}

function isInsideBlock(lines, lineNum, blockType) {
  let depth = 0
  
  for (let i = lineNum - 1; i >= 0; i--) {
    const line = lines[i]
    if (line.match(new RegExp(`^${blockType}\\s+`))) {
      return depth > 0
    }
    for (const char of line) {
      if (char === '}') depth++
      if (char === '{') depth--
    }
  }
  
  return false
}

function isInsidePythonClass(lines, lineNum) {
  const baseIndent = lines[lineNum].search(/\S/)
  
  for (let i = lineNum - 1; i >= 0; i--) {
    const line = lines[i]
    if (line.trim() === '') continue
    
    const indent = line.search(/\S/)
    if (indent < baseIndent && line.match(/^class\s+/)) {
      return true
    }
    if (indent === 0) {
      return false
    }
  }
  
  return false
}
