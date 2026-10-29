import { invoke } from '@tauri-apps/api/core';
import { getCurrentWindow } from '@tauri-apps/api/window';

const currentWindow = getCurrentWindow();
const launcher = Object.freeze({
  getAppState: () => invoke('get_app_state'),
  loadSettings: () => invoke('get_settings'),
  getDefaults: () => invoke('get_defaults'),
  saveSettings: (settings) => invoke('save_launcher_settings', { settings }),
  launchGame: () => invoke('launch_game'),
  openIni: () => invoke('open_ini'),
  windowAction: (action) => action === 'minimize' ? currentWindow.minimize() : currentWindow.close(),
});

const categories = [
  { id: 'display', title: 'Display', icon: '◈' },
  { id: 'graphics', title: 'Graphics', icon: '◇' },
  { id: 'controller', title: 'Controller', icon: '◉' },
  { id: 'audio', title: 'Audio', icon: '♫' },
  { id: 'keyboardMouse', title: 'Keyboard/Mouse', icon: '⌘' },
  { id: 'multiplayer', title: 'Multiplayer', icon: '◎' },
  { id: 'gameplay', title: 'Gameplay', icon: '⚔' },
];

const keyboardBindings = [
  ['keyDpadUp', 'D-pad up'], ['keyDpadUpAlt', 'D-pad up (alternate)'],
  ['keyDpadRight', 'D-pad right'], ['keyDpadRightAlt', 'D-pad right (alternate)'],
  ['keyDpadDown', 'D-pad down'], ['keyDpadDownAlt', 'D-pad down (alternate)'],
  ['keyDpadLeft', 'D-pad left'], ['keyDpadLeftAlt', 'D-pad left (alternate)'],
  ['keyButtonA', 'A button'], ['keyButtonAAlt', 'A button (confirm)'],
  ['keyButtonB', 'B button'], ['keyButtonBAlt', 'B button (back)'],
  ['keyButtonX', 'X button'], ['keyButtonXAlt', 'X button (alternate)'],
  ['keyButtonY', 'Y button'], ['keyButtonYAlt', 'Y button (alternate)'],
  ['keyButtonL', 'L button'], ['keyButtonLAlt', 'L button (alternate)'],
  ['keyButtonR', 'R button'], ['keyButtonRAlt', 'R button (alternate)'],
  ['keyButtonZl', 'ZL button'], ['keyButtonZlAlt', 'ZL button (alternate)'],
  ['keyButtonZr', 'ZR button'], ['keyButtonZrAlt', 'ZR button (alternate)'],
  ['keyButtonPlus', 'Plus button'],
  ['keyButtonPlusAlt', 'Plus button (alternate)'], ['keyButtonPlusAlt2', 'Plus button (optional)'],
  ['keyButtonMinus', 'Minus button'], ['keyButtonMinusAlt', 'Minus button (alternate)'],
  ['keyButtonLeftStick', 'Left stick click'], ['keyButtonLeftStickAlt', 'Left stick click (alternate)'],
  ['keyButtonRightStick', 'Right stick click'], ['keyButtonRightStickAlt', 'Right stick click (alternate)'],
  ['keyStickUp', 'Move up'], ['keyStickLeft', 'Move left'], ['keyStickDown', 'Move down'],
  ['keyStickRight', 'Move right'],
];

const mouseBindings = [
  ['mouseLeftButton', 'Mouse left button'], ['mouseMiddleButton', 'Mouse middle button'],
  ['mouseRightButton', 'Mouse right button'], ['mouseX1Button', 'Mouse side button X1'],
  ['mouseX2Button', 'Mouse side button X2'],
];

const movementBindingStart = keyboardBindings.findIndex(([id]) => id === 'keyStickUp');

const controlIslands = [
  {
    title: 'Mouse',
    fields: ['mouseSensitivity', 'cameraSmoothing', 'cameraInvertHorizontal', 'cameraInvertVertical',
      ...mouseBindings.map(([id]) => id)],
  },
  {
    title: 'Keyboard',
    groups: [
      {
        title: 'D-pad',
        fields: keyboardBindings.slice(0, 8).map(([id]) => id),
      },
      {
        title: 'Buttons',
        fields: keyboardBindings.slice(8, movementBindingStart).map(([id]) => id),
      },
      {
        title: 'Movement',
        fields: keyboardBindings.slice(movementBindingStart).map(([id]) => id),
      },
    ],
  },
];

const categoryFields = {
  display: ['mode', 'resolution', 'renderResolution', 'aspectRatio', 'fov', 'uiAnchor'],
  graphics: [
    'msaa', 'fxaa', 'ssao', 'fsr', 'fsrUpscaler', 'fsrQuality', 'fsrSharpness',
    'fsrObjectMotion', 'fsrReactive', 'fsrAnimSurf', 'drawDistance', 'fps', 'vsync', 'shaderPrewarm',
  ],
  controller: ['faceButtons', 'backgroundInput', 'rumble', 'comboInputWindow'],
  audio: ['muteInBackground'],
  keyboardMouse: [],
  multiplayer: ['port', 'join', 'syncSmallMonsters'],
  gameplay: ['villageQuestsInHub', 'hubHp', 'bonusContent', 'hp1', 'hp2', 'hp3', 'hp4', 'quests'],
};

const fieldInfo = {
  mode: { label: 'Display mode', kind: 'select', options: ['Borderless', 'Fullscreen', 'Windowed'], help: 'Choose how the game occupies your display.' },
  resolution: {
    label: 'Display resolution',
    kind: 'preset',
    options: ['Desktop', '1280x720', '1600x900', '1920x1080', '2560x1080', '2560x1440', '3440x1440', '3840x2160', '5120x1440'],
    customType: 'text',
    placeholder: 'e.g. 1920x1080',
    help: 'Choose a common display size, or select Custom to enter another WxH resolution.',
  },
  renderResolution: {
    label: '3D render resolution',
    kind: 'preset',
    options: ['Auto', '1280x720', '1600x900', '1920x1080', '2560x1440', '3840x2160'],
    customType: 'text',
    placeholder: 'e.g. 1920x1080',
    help: 'Auto picks the largest resolution that fits your display.',
  },
  aspectRatio: {
    label: 'Aspect ratio',
    kind: 'preset',
    options: ['Auto', '16:9', '16:10', '21:9', '32:9'],
    customType: 'text',
    placeholder: 'e.g. 2.0 or 21:9',
    help: 'Choose a common screen shape, or select Custom to enter another ratio.',
  },
  fov: {
    label: 'Field of view',
    kind: 'preset',
    options: ['0.8', '0.9', '1.0', '1.1', '1.2', '1.3', '1.5', '2.0'],
    customType: 'number',
    min: 0.5,
    max: 2,
    step: 0.1,
    help: '1.0 uses the game default.',
  },
  uiAnchor: {
    label: 'HUD position on ultrawide',
    kind: 'select',
    options: ['Center', '21:9', '32:9', 'Edges'],
    help: 'Choose how far the HUD spreads toward the sides of an ultrawide display.',
  },
  msaa: { label: 'MSAA', kind: 'select', options: ['Off', '2x', '4x', '8x'], help: 'Multisample anti-aliasing.' },
  fxaa: { label: 'FXAA', kind: 'toggle', help: 'A lightweight post-processing anti-aliasing pass.' },
  ssao: { label: 'SSAO', kind: 'toggle', help: 'Experimental depth-aware contact shadows, including around skybox edges. Needs scene depth, so flat 2D loading screens are unaffected; may reduce performance.' },
  fsr: { label: 'FSR', kind: 'toggle', help: 'Temporal upscaling. Renders the 3D scene at a lower resolution and reconstructs it at the output resolution; the HUD stays full resolution. MSAA and FXAA are disabled while FSR is active.' },
  fsrUpscaler: { label: 'Upscaler', kind: 'select', options: [{ label: 'FSR 3.1', value: 'FSR3' }, { label: 'FSR 4', value: 'FSR4' }], help: 'FSR 4 requires a supported RDNA 4 GPU and compatible AMD driver on Windows. FSR 3.1 is used if FSR 4 cannot start.' },
  fsrQuality: { label: 'FSR quality', kind: 'select', options: ['NativeAA', 'Quality', 'Balanced', 'Performance', 'UltraPerformance'], help: 'Higher quality renders closer to the output resolution; Performance modes render faster but look softer.' },
  fsrSharpness: { label: 'FSR sharpness', kind: 'number', min: 0, max: 1, step: 0.1, help: 'Sharpening strength, from off to strongest.' },
  fsrObjectMotion: { label: 'Object motion vectors', kind: 'toggle', help: 'Tracks moving characters and monsters to reduce temporal ghosting. Can be disabled if it causes issues.' },
  fsrReactive: { label: 'Reactive mask strength', kind: 'number', min: 0, max: 1, step: 0.1, help: 'Helps preserve detail on moving objects; set to 0 to disable.' },
  fsrAnimSurf: { label: 'Animated surface detection', kind: 'toggle', help: 'Detects texture-animated surfaces such as water to reduce ghosting.' },
  drawDistance: {
    label: 'Draw distance',
    kind: 'select',
    options: [
      { label: '0 (No limit)', value: '0' },
      ...Array.from({ length: 10 }, (_, index) => String(index + 1)),
    ],
    help: 'Controls how far away people in villages and Hubs are drawn. 1 is the game default; 0 means no limit.',
  },
  fps: {
    label: 'Frame rate limit',
    kind: 'preset',
    options: ['30', '60', '75', '90', '120', '144', '165', '180', '240'],
    customType: 'number',
    min: 30,
    max: 240,
    step: 1,
    suffix: 'FPS',
  },
  vsync: { label: 'VSync', kind: 'toggle', help: 'Synchronize frames with your display refresh.' },
  shaderPrewarm: { label: 'Shader prewarm', kind: 'toggle', help: 'Translate game shaders in the background at startup to reduce first-use stutter. Initial startup takes longer and uses additional shader cache space.' },
  faceButtons: {
    label: 'Face button mapping',
    kind: 'select',
    options: [
      { label: 'Switch layout (by position)', value: 'Position' },
      { label: 'Match printed labels', value: 'Label' },
    ],
    help: 'Switch layout: the bottom button acts as B and the right button as A, whatever is printed on them. Match printed labels: an Xbox A button acts as A.',
  },
  backgroundInput: {
    label: 'Controller works in background',
    kind: 'toggle',
    help: 'Keep controller input active while another window is focused.',
  },
  rumble: {
    label: 'Controller rumble',
    kind: 'toggle',
    help: "Send the game's vibration to the connected controller.",
  },
  comboInputWindow: {
    label: 'Combo input window',
    kind: 'select',
    options: ['Lenient', 'Strict'],
    help: 'Lenient allows more time between inputs in a combo; Strict requires more precise timing.',
  },
  muteInBackground: {
    label: 'Mute when in background',
    kind: 'toggle',
    help: 'Mute game audio while its window is not focused.',
  },
  mouseSensitivity: {
    label: 'Mouse sensitivity',
    kind: 'preset',
    options: ['0.5', '0.75', '1.0', '1.25', '1.5', '2.0', '3.0'],
    customType: 'number',
    min: 0.1,
    max: 10,
    step: 0.1,
  },
  cameraSmoothing: {
    label: 'Camera response',
    kind: 'preset',
    options: ['0.2', '0.35', '0.5', '0.7', '1.0'],
    customType: 'number',
    min: 0.05,
    max: 1,
    step: 0.05,
    help: 'Lower values smooth movement more but respond more slowly; 1.0 responds immediately.',
  },
  cameraInvertHorizontal: { label: 'Invert horizontal look', kind: 'toggle' },
  cameraInvertVertical: { label: 'Invert vertical look', kind: 'toggle' },
  mouseButton: {
    kind: 'select',
    options: [
      'None', 'A', 'B', 'X', 'Y', 'L Stick', 'R Stick', 'L', 'R', 'ZL', 'ZR', 'Plus', 'Minus',
      'D-pad Left', 'D-pad Up', 'D-pad Right', 'D-pad Down',
    ],
    help: 'Choose the Switch button this mouse button activates. The click that captures the mouse is not sent to the game.',
  },
  port: { label: 'Multiplayer port', kind: 'number', min: 1, max: 65535 },
  join: { label: 'Remote hosts', kind: 'text', help: 'Comma-separated addresses of players to join.' },
  syncSmallMonsters: { label: 'Synchronize small monsters', kind: 'toggle', help: 'Keep small monster state synchronized in multiplayer.' },
  villageQuestsInHub: { label: 'Village quests at the Hub', kind: 'toggle' },
  hubHp: { label: 'Village quest HP multiplier', kind: 'number', min: 0.05, max: 20 },
  bonusContent: { label: 'Copy bonus content', kind: 'toggle', help: 'Copy bonus content from a donor save.' },
  hp1: { label: '1 hunter', kind: 'number', min: 0.05, max: 20 },
  hp2: { label: '2 hunters', kind: 'number', min: 0.05, max: 20 },
  hp3: { label: '3 hunters', kind: 'number', min: 0.05, max: 20 },
  hp4: { label: '4 hunters', kind: 'number', min: 0.05, max: 20 },
  quests: { label: 'Per-quest HP multipliers', kind: 'quests', help: 'One multiplier or four comma-separated player-count values per quest. Example: Quest123 = 1.0, 1.2, 1.5, 2.0' },
};
for (const [id, label] of keyboardBindings) {
  fieldInfo[id] = {
    label,
    kind: 'key',
    help: id === 'keyButtonPlusAlt2'
      ? 'Optional. Press Delete while assigning to leave this binding empty.'
      : 'Click, then press a key to assign it.',
  };
}
for (const [id, label] of mouseBindings) {
  fieldInfo[id] = { ...fieldInfo.mouseButton, label };
}

const elements = Object.fromEntries([
  'home-screen', 'settings-screen', 'settings-nav', 'category-title',
  'field-grid', 'settings-form', 'form-error', 'unsaved-indicator', 'toast', 'build-status',
  'build-status-text', 'reset-button', 'save-button', 'save-play-button',
].map((id) => [id, document.getElementById(id)]));

let settings = null;
let activeCategory = 'display';
let isDirty = false;
let toastTimer = null;
let listeningForBinding = null;

function makeElement(tag, className, text) {
  const element = document.createElement(tag);
  if (className) element.className = className;
  if (text !== undefined) element.textContent = text;
  return element;
}

function showToast(message, isError = false) {
  clearTimeout(toastTimer);
  elements.toast.textContent = message;
  elements.toast.classList.toggle('error', isError);
  elements.toast.classList.add('visible');
  toastTimer = setTimeout(() => elements.toast.classList.remove('visible'), 3400);
}

function confirmAction(title, message, actionLabel) {
  return new Promise((resolve) => {
    const dialog = document.getElementById('confirm-dialog');
    const accept = document.getElementById('confirm-accept');
    const cancel = document.getElementById('confirm-cancel');
    document.getElementById('confirm-title').textContent = title;
    document.getElementById('confirm-copy').textContent = message;
    accept.textContent = actionLabel;
    dialog.hidden = false;
    accept.focus();

    const finish = (accepted) => {
      dialog.hidden = true;
      accept.removeEventListener('click', onAccept);
      cancel.removeEventListener('click', onCancel);
      resolve(accepted);
    };
    const onAccept = () => finish(true);
    const onCancel = () => finish(false);
    accept.addEventListener('click', onAccept);
    cancel.addEventListener('click', onCancel);
  });
}

function setDirty(value) {
  isDirty = value;
  elements['unsaved-indicator'].hidden = !value;
}

function createNav() {
  for (const category of categories) {
    const button = makeElement('button', 'nav-item');
    button.type = 'button';
    button.dataset.category = category.id;
    button.append(
      makeElement('span', 'nav-icon', category.icon),
      makeElement('span', '', category.title),
    );
    button.addEventListener('click', () => {
      readVisibleFields();
      renderCategory(category.id);
    });
    elements['settings-nav'].append(button);
  }
}

function createField(id) {
  const info = fieldInfo[id];
  const label = makeElement('label', `field-card${id === 'quests' ? ' wide' : ''}`);
  label.htmlFor = `setting-${id}`;
  label.append(makeElement('span', 'field-label', info.label));
  let control;
  if (info.kind === 'key') {
    control = makeElement('button', 'field-control key-binding', 'Press to assign');
    control.type = 'button';
    control.addEventListener('click', () => {
      if (listeningForBinding === control) {
        listeningForBinding = null;
        writeVisibleFields();
      } else {
        listeningForBinding = control;
        control.textContent = 'Press a key…';
      }
    });
  } else if (info.kind === 'select' || info.kind === 'toggle') {
    control = makeElement('select', 'field-control');
    const choices = info.kind === 'toggle' ? ['On', 'Off'] : info.options;
    for (const choice of choices) {
      if (typeof choice === 'string') {
        control.add(new Option(choice, choice));
      } else {
        control.add(new Option(choice.label, choice.value));
      }
    }
  } else if (info.kind === 'preset') {
    control = makeElement('div', 'preset-control');
    control.dataset.controlKind = 'preset';
    const select = makeElement('select', 'field-control');
    select.id = `setting-${id}`;
    for (const choice of info.options) select.add(new Option(choice, choice));
    select.add(new Option('Custom…', '__custom__'));
    const custom = makeElement('input', 'field-control');
    custom.type = info.customType;
    custom.placeholder = info.placeholder ?? 'Enter a value';
    if (info.min !== undefined) {
      custom.min = String(info.min);
      custom.max = String(info.max);
      custom.step = String(info.step ?? 1);
      if (info.customType === 'number') custom.inputMode = 'decimal';
    }
    custom.hidden = true;
    select.addEventListener('change', () => {
      custom.hidden = select.value !== '__custom__';
      control.classList.toggle('custom-active', !custom.hidden);
    });
    control.append(select, custom);
  } else if (info.kind === 'quests') {
    control = makeElement('textarea', 'field-control');
    control.spellcheck = false;
    control.placeholder = 'Quest123 = 1.0\nQuest456 = 1, 1.2, 1.5, 2';
  } else {
    control = makeElement('input', 'field-control');
    control.type = info.kind === 'number' ? 'number' : 'text';
    if (info.kind === 'number') {
      control.min = String(info.min);
      control.max = String(info.max);
      control.step = String(info.step ?? 1);
      control.inputMode = 'decimal';
    }
  }
  if (info.kind !== 'preset') control.id = `setting-${id}`;
  control.dataset.setting = id;
  control.addEventListener('input', () => setDirty(true));
  control.addEventListener('change', () => setDirty(true));
  label.append(control);
  if (info.suffix || info.min !== undefined) {
    const range = info.suffix
      ? `${info.min}–${info.max} ${info.suffix}`
      : `${info.min}–${info.max}`;
    label.append(makeElement('span', 'field-help', info.help ?? range));
  } else if (info.help) {
    label.append(makeElement('span', 'field-help', info.help));
  }
  return label;
}

function createControlIsland(title, fieldIds, groups = null) {
  const island = makeElement('section', 'control-island');
  island.append(makeElement('h3', 'control-island-title', title));
  if (groups) {
    for (const group of groups) {
      const subsection = makeElement('div', 'control-subsection');
      subsection.append(makeElement('h4', 'control-subsection-title', group.title));
      const grid = makeElement('div', 'control-island-grid');
      for (const id of group.fields) grid.append(createField(id));
      subsection.append(grid);
      island.append(subsection);
    }
  } else {
    const grid = makeElement('div', 'control-island-grid');
    for (const id of fieldIds) grid.append(createField(id));
    island.append(grid);
  }
  return island;
}

function renderCategory(categoryId) {
  listeningForBinding = null;
  activeCategory = categoryId;
  const category = categories.find((item) => item.id === categoryId);
  elements['category-title'].textContent = category.title;
  for (const item of elements['settings-nav'].querySelectorAll('.nav-item')) {
    const active = item.dataset.category === categoryId;
    item.classList.toggle('active', active);
    item.setAttribute('aria-current', active ? 'page' : 'false');
  }
  const fieldIds = categoryFields[categoryId];
  elements['field-grid'].replaceChildren();
  if (categoryId === 'gameplay') {
    for (const id of fieldIds.slice(0, 3)) elements['field-grid'].append(createField(id));
    const hpGroup = makeElement('div', 'hp-grid wide');
    for (const id of fieldIds.slice(3, 7)) hpGroup.append(createField(id));
    elements['field-grid'].append(hpGroup, createField('quests'));
  } else if (categoryId === 'controller') {
    elements['field-grid'].append(createControlIsland('Controller', fieldIds));
  } else if (categoryId === 'keyboardMouse') {
    for (const island of controlIslands) {
      elements['field-grid'].append(createControlIsland(island.title, island.fields, island.groups));
    }
  } else {
    for (const id of fieldIds) elements['field-grid'].append(createField(id));
  }
  writeVisibleFields();
}

function writeVisibleFields() {
  if (!settings) return;
  for (const control of elements['field-grid'].querySelectorAll('[data-setting]')) {
    const id = control.dataset.setting;
    if (control.dataset.controlKind === 'preset') {
      const select = control.querySelector('select');
      const custom = control.querySelector('input');
      const value = settings[id] ?? '';
      const hasPreset = fieldInfo[id].options.includes(value);
      select.value = hasPreset ? value : '__custom__';
      custom.value = hasPreset ? '' : value;
      custom.hidden = hasPreset;
      control.classList.toggle('custom-active', !hasPreset);
    } else if (id === 'quests') {
      control.value = settings.quests.map(({ id: questId, value }) => `Quest${questId} = ${value}`).join('\n');
    } else if (control instanceof HTMLButtonElement) {
      control.dataset.boundKey = settings[id] ?? '';
      control.textContent = listeningForBinding === control ? 'Press a key…' : settings[id] ?? '';
    } else if (id === 'drawDistance') {
      const value = settings[id] ?? '';
      if (value && !Array.from(control.options).some((option) => option.value === value)) {
        control.add(new Option(`${value} (current setting)`, value));
      }
      control.value = value;
    } else {
      control.value = settings[id] ?? '';
    }
  }
}

function readQuestLines(text) {
  const quests = [];
  for (const line of text.split(/\r?\n/)) {
    if (!line.trim()) continue;
    const match = /^\s*Quest(\d+)\s*=\s*(.*?)\s*$/i.exec(line);
    if (match) quests.push({ id: Number(match[1]), value: match[2] });
    else quests.push({ id: 0, value: line });
  }
  return quests;
}

function readVisibleFields() {
  if (!settings) return;
  for (const control of elements['field-grid'].querySelectorAll('[data-setting]')) {
    const id = control.dataset.setting;
    if (control.dataset.controlKind === 'preset') {
      const select = control.querySelector('select');
      settings[id] = select.value === '__custom__'
        ? control.querySelector('input').value
        : select.value;
    } else if (id === 'quests') {
      settings[id] = readQuestLines(control.value);
    } else if (control instanceof HTMLButtonElement) {
      settings[id] = control.dataset.boundKey;
    } else {
      settings[id] = control.value;
    }
  }
}

function keyNameFromCode(code) {
  if (/^Key[A-Z]$/.test(code)) return code.slice(3);
  if (/^Digit[0-9]$/.test(code)) return code.slice(5);
  if (/^F([1-9]|1[0-2])$/.test(code)) return code;
  if (/^Numpad[0-9]$/.test(code)) return `Keypad ${code.slice(6)}`;
  const names = {
    ArrowUp: 'Up', ArrowDown: 'Down', ArrowLeft: 'Left', ArrowRight: 'Right',
    Escape: 'Escape', Enter: 'Return', NumpadEnter: 'Keypad Enter', Space: 'Space', Tab: 'Tab',
    Backspace: 'Backspace', ShiftLeft: 'Left Shift', ShiftRight: 'Right Shift',
    ControlLeft: 'Left Ctrl', ControlRight: 'Right Ctrl', AltLeft: 'Left Alt', AltRight: 'Right Alt',
    CapsLock: 'CapsLock', Insert: 'Insert',     Delete: 'None', Home: 'Home', End: 'End',
    PageUp: 'PageUp', PageDown: 'PageDown', Minus: '-', Equal: '=', BracketLeft: '[',
    BracketRight: ']', Backslash: '\\', Semicolon: ';', Quote: "'", Backquote: '`',
    Comma: ',', Period: '.', Slash: '/', NumpadAdd: 'Keypad +', NumpadSubtract: 'Keypad -',
    NumpadMultiply: 'Keypad *', NumpadDivide: 'Keypad /', NumpadDecimal: 'Keypad .',
  };
  return names[code] ?? null;
}

document.addEventListener('keydown', (event) => {
  if (!listeningForBinding) return;
  event.preventDefault();
  event.stopImmediatePropagation();
  const name = keyNameFromCode(event.code);
  if (!name) {
    showToast('That key is not supported for bindings.', true);
    return;
  }
  const control = listeningForBinding;
  listeningForBinding = null;
  control.dataset.boundKey = name;
  control.textContent = name;
  elements['form-error'].hidden = true;
  setDirty(true);
});

function showSaveError(result) {
  const messages = result.validation ? Object.values(result.validation) : [result.message];
  elements['form-error'].textContent = messages.join(' ');
  elements['form-error'].hidden = false;
}

async function saveCurrentSettings(launchAfterSave = false) {
  readVisibleFields();
  elements['form-error'].hidden = true;
  try {
    const result = await launcher.saveSettings(settings);
    if (!result.ok) {
      showSaveError(result);
      return false;
    }
    setDirty(false);
    showToast('Settings saved to mhgurecomp.ini.');
    if (launchAfterSave) await launchGame();
    return true;
  } catch (error) {
    showToast(error.message, true);
    return false;
  }
}

async function launchGame() {
  try {
    await launcher.launchGame();
    await launcher.windowAction('close');
  } catch (error) {
    showToast(error.message, true);
  }
}

function switchScreen(screen) {
  listeningForBinding = null;
  const showSettings = screen === 'settings';
  elements['home-screen'].hidden = showSettings;
  elements['settings-screen'].hidden = !showSettings;
  if (showSettings) {
    launcher.loadSettings().then((loaded) => {
      settings = loaded;
      renderCategory(activeCategory);
      setDirty(false);
    }).catch((error) => showToast(error.message, true));
  }
}

function applyTheme(theme) {
  document.documentElement.dataset.theme = theme;
}

function setupActions() {
  document.getElementById('play-button').addEventListener('click', launchGame);
  document.getElementById('settings-button').addEventListener('click', () => switchScreen('settings'));
  document.getElementById('exit-button').addEventListener('click', () => launcher.windowAction('close'));
  document.getElementById('back-button').addEventListener('click', async () => {
    if (isDirty && !await confirmAction('Leave settings?', 'Your changes have not been saved.', 'LEAVE')) return;
    switchScreen('home');
  });
  document.getElementById('open-ini-button').addEventListener('click', async () => {
    try {
      await launcher.openIni();
    } catch (error) {
      showToast(error.message, true);
    }
  });
  document.getElementById('settings-form').addEventListener('submit', (event) => event.preventDefault());
  elements['field-grid'].addEventListener('input', () => {
    elements['form-error'].hidden = true;
  });
  elements['save-button'].addEventListener('click', () => saveCurrentSettings());
  elements['save-play-button'].addEventListener('click', () => saveCurrentSettings(true));
  elements['reset-button'].addEventListener('click', async () => {
    if (!await confirmAction('Restore defaults?', 'All launcher settings will return to their default values.', 'RESTORE')) return;
    launcher.getDefaults().then((defaults) => {
      settings = defaults;
      renderCategory(activeCategory);
      setDirty(true);
    }).catch((error) => showToast(error.message, true));
  });
  for (const button of document.querySelectorAll('[data-window-action]')) {
    button.addEventListener('click', () => launcher.windowAction(button.dataset.windowAction));
  }
  document.querySelector('.titlebar').addEventListener('mousedown', (event) => {
    if (event.button === 0 && !event.target.closest('.window-action')) currentWindow.startDragging();
  });
}

async function initialize() {
  createNav();
  setupActions();
  try {
    const state = await launcher.getAppState();
    const themePreference = window.matchMedia('(prefers-color-scheme: light)');
    applyTheme(themePreference.matches ? 'light' : 'dark');
    themePreference.addEventListener('change', (event) => applyTheme(event.matches ? 'light' : 'dark'));
    document.documentElement.dataset.background = state.background;
    const status = document.querySelector('.build-status');
    if (!state.gameExists) {
      status.classList.add('missing');
      status.hidden = false;
      elements['build-status-text'].textContent = 'GAME FILES NOT FOUND';
      document.getElementById('play-button').title = 'mhgurecomp.exe must be next to the launcher.';
    } else {
      elements['build-status-text'].textContent = 'GAME FILES READY';
    }
  } catch (error) {
    showToast(error.message, true);
  }
}

initialize();
