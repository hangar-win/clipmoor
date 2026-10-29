import type { ComponentPropsWithoutRef } from 'react';

import * as stylex from '@stylexjs/stylex';

import { border, fontSize, fontWeight, lineHeight, opacity, size as dim, space } from '@/lib/constants.stylex';
import { colors } from '@/lib/tokens.stylex';
import { ink, radius } from '@/tokens.stylex';

export type ButtonVariant = 'secondary' | 'primary' | 'danger' | 'dangerFill';
export type ButtonSize = 'sm' | 'md';

type ButtonProps = Omit<ComponentPropsWithoutRef<'button'>, 'className' | 'style'> & { variant?: ButtonVariant; size?: ButtonSize; style?: stylex.StyleXStyles };

export function Button({ variant = 'secondary', size = 'md', style, ...props }: ButtonProps) {
  return <button type="button" {...props} {...stylex.props(styles.base, styles[variant], size === 'sm' && styles.sm, style)} />;
}

const styles = stylex.create({
  base: {
    alignItems: 'center',
    backgroundColor: { default: 'transparent', ':hover': { default: ink.strip, ':disabled': 'transparent' } },
    borderColor: ink.frame,
    borderRadius: radius.control,
    borderStyle: 'solid',
    borderWidth: border.stroke,
    color: ink.textControl,
    cursor: 'default',
    display: 'inline-flex',
    flexShrink: 0,
    fontSize: fontSize.titleStrong,
    fontWeight: fontWeight.titleStrong,
    gap: space.s6,
    height: dim.controlMd,
    justifyContent: 'center',
    lineHeight: lineHeight.titleStrong,
    opacity: { default: 1, ':disabled': opacity.disabled },
    outlineColor: ink.focusRing,
    outlineOffset: border.offsetFocus,
    outlineStyle: { default: 'none', ':focus-visible': 'solid' },
    outlineWidth: border.strokeRing,
    paddingInline: space.s12,
    whiteSpace: 'nowrap',
  },
  secondary: {},
  primary: {
    backgroundColor: ink.active,
    borderColor: ink.activeEdge,
    color: ink.textOnActive,
    outlineColor: ink.textOnActive,
    outlineOffset: `calc(2 * ${border.offsetFocus})`,
  },
  danger: {
    borderColor: colors.settingsDanger,
    color: colors.settingsDanger,
  },
  dangerFill: {
    backgroundColor: colors.settingsDanger,
    borderColor: colors.settingsDanger,
    color: colors.settingsOnDanger,
    outlineColor: colors.settingsOnDanger,
    outlineOffset: `calc(2 * ${border.offsetFocus})`,
  },
  sm: {
    borderColor: 'transparent',
    color: ink.textMuted,
    fontSize: fontSize.control,
    fontWeight: fontWeight.control,
    height: dim.controlSm,
    lineHeight: lineHeight.control,
    opacity: { default: 1, ':disabled': opacity.idle },
    paddingInline: space.s8,
  },
});
