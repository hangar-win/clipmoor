import * as stylex from '@stylexjs/stylex';

export const space = stylex.defineConsts({
  s1: '1px',
  s2: '2px',
  s3: '3px',
  s4: '4px',
  s5: '5px',
  s6: '6px',
  s8: '8px',
  s9: '9px',
  s10: '10px',
  s12: '12px',
  s14: '14px',
  s16: '16px',
  s24: '24px',
  s28: '28px',
  s32: '32px',
});

export const size = stylex.defineConsts({
  controlSm: '24px',
  controlSegment: '28px',
  controlIcon: '28px',
  controlMd: '32px',
  controlLg: '38px',
  itemH: '76px',
  stripH: '24px',
  bandW: '10px',
  bandSlimW: '4px',
  compactRowH: '36px',
  collapsedBarH: '48px',
  tabKey: '8px',
  thumbH: '40px',
  iconSm: '12px',
  iconMd: '16px',
  menuMinW: '184px',
  menuMaxW: '260px',
  switchW: '36px',
  fieldShortcutMin: '152px',
  railW: '280px',
  panelW: '448px',
  panelH: '640px',
  shelfW: '920px',
  shelfH: '260px',
  shelfCardH: '180px',
  shelfMediaH: '48px',
  shelfSearchW: '200px',
  paletteW: '680px',
  paletteH: '420px',
  previewTextMaxW: '720px',
  previewTextMaxH: '480px',
  previewSwatch: '160px',
  paletteListW: '300px',
  queryH: '56px',
  settingsW: '1080px',
  settingsH: '680px',
  settingsContentMax: '736px',
  settingsPreviewH: '260px',
  switchH: '20px',
  switchThumb: '16px',
});

export const border = stylex.defineConsts({
  stroke: '1px',
  strokeRing: '2px',
  offsetSelection: '1px',
  offsetFocus: '-2px',
  offsetFocusSwitch: '2px',
});

export const fontSize = stylex.defineConsts({
  label: '11px',
  keycap: '11px',
  control: '12px',
  controlStrong: '12px',
  title: '13px',
  titleStrong: '13px',
  body: '14px',
  heading: '14px',
  query: '18px',
  code: '13px',
  settingsHint: '12px',
  settingsCaption: '12px',
  settingsBody: '14px',
  settingsBodyStrong: '14px',
  settingsTitle: '18px',
});

export const lineHeight = stylex.defineConsts({
  label: '16px',
  keycap: '14px',
  control: '16px',
  controlStrong: '16px',
  title: '18px',
  titleStrong: '18px',
  body: '19px',
  heading: '19px',
  query: '24px',
  code: '19px',
  settingsHint: '16px',
  settingsCaption: '16px',
  settingsBody: '20px',
  settingsBodyStrong: '20px',
  settingsTitle: '28px',
});

export const fontWeight = stylex.defineConsts({
  label: '400',
  keycap: '400',
  control: '400',
  controlStrong: '600',
  title: '400',
  titleStrong: '600',
  body: '400',
  heading: '600',
  query: '400',
  code: '400',
  settingsHint: '400',
  settingsCaption: '600',
  settingsBody: '400',
  settingsBodyStrong: '500',
  settingsTitle: '600',
});

export const letterSpacing = stylex.defineConsts({
  label: '0.02em',
});

export const duration = stylex.defineConsts({
  caretBlink: '1060ms',
  previewDelay: '300ms',
  fast: '150ms',
  normal: '200ms',
});

export const easing = stylex.defineConsts({
  out: 'cubic-bezier(0.23, 1, 0.32, 1)',
});

export const scale = stylex.defineConsts({
  popupEnter: '0.97',
});

export const opacity = stylex.defineConsts({
  disabled: '0.5',
  idle: '0.6',
});

export const z = stylex.defineConsts({
  raised: '1',
  popup: '50',
});
