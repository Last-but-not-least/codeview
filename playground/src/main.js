import './style.css'
import { examples } from './examples.js'
import { parseCode } from './parser.js'

const languageSelect = document.getElementById('language')
const exampleSelect = document.getElementById('example')
const expandedCheckbox = document.getElementById('expanded')
const inputArea = document.getElementById('input')
const outputArea = document.getElementById('output')

let currentLanguage = 'rust'
let currentExampleIndex = 0

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
  const code = inputArea.value
  const expanded = expandedCheckbox.checked
  const result = parseCode(code, currentLanguage, expanded)
  outputArea.textContent = result
}

// Event listeners
languageSelect.addEventListener('change', updateExamples)
exampleSelect.addEventListener('change', loadExample)
expandedCheckbox.addEventListener('change', updateOutput)
inputArea.addEventListener('input', updateOutput)

// Initialize
updateExamples()
