import './style.css'
import { examples } from './examples.js'
import { parseCode } from './tsParser.js'

const languageSelect = document.getElementById('language')
const exampleSelect = document.getElementById('example')
const symbolsInput = document.getElementById('symbols')
const pubCheckbox = document.getElementById('pub-only')
const fnsCheckbox = document.getElementById('fns-only')
const typesCheckbox = document.getElementById('types-only')
const noTestsCheckbox = document.getElementById('no-tests')
const jsonCheckbox = document.getElementById('json-output')
const statsCheckbox = document.getElementById('stats-output')
const inputArea = document.getElementById('input')
const outputArea = document.getElementById('output')

let currentLanguage = 'rust'
let currentExampleIndex = 0
let parseTimeout = null

function updateExamples() {
  const lang = languageSelect.value
  const langExamples = examples[lang]
  
  exampleSelect.innerHTML = langExamples
    .map((ex, i) => `<option value="${i}">${ex.name}</option>`)
    .join('')
  
  currentLanguage = lang
  currentExampleIndex = 0
  loadExample()
}

function loadExample() {
  const exampleIndex = parseInt(exampleSelect.value)
  const example = examples[currentLanguage][exampleIndex]
  
  currentExampleIndex = exampleIndex
  inputArea.value = example.code
  updateOutput()
}

function updateOutput() {
  // Debounce parsing to avoid lag while typing
  if (parseTimeout) {
    clearTimeout(parseTimeout)
  }
  
  parseTimeout = setTimeout(async () => {
    const code = inputArea.value
    const options = {
      symbols: symbolsInput.value,
      pubOnly: pubCheckbox.checked,
      fnsOnly: fnsCheckbox.checked,
      typesOnly: typesCheckbox.checked,
      noTests: noTestsCheckbox.checked,
      json: jsonCheckbox.checked,
      stats: statsCheckbox.checked
    }
    
    const result = await parseCode(code, currentLanguage, options)
    outputArea.textContent = result
  }, 300)
}

// Event listeners
languageSelect.addEventListener('change', updateExamples)
exampleSelect.addEventListener('change', loadExample)
symbolsInput.addEventListener('input', updateOutput)
pubCheckbox.addEventListener('change', updateOutput)
fnsCheckbox.addEventListener('change', updateOutput)
typesCheckbox.addEventListener('change', updateOutput)
noTestsCheckbox.addEventListener('change', updateOutput)
jsonCheckbox.addEventListener('change', updateOutput)
statsCheckbox.addEventListener('change', updateOutput)
inputArea.addEventListener('input', updateOutput)

// Initialize
updateExamples()
