import * as stylex from '@stylexjs/stylex';

import { border, size } from './constants.stylex';
import { ink } from '../tokens.stylex';

export type StyleXStyles = stylex.StyleXStyles;

const recipes = stylex.create({
  boxShadow: (value: string) => ({ boxShadow: value }),
});

export interface RingOptions {
  width?: string;
  color?: string;
}

export const ring = ({ width = border.stroke, color = ink.frame }: RingOptions = {}) => recipes.boxShadow(`0 0 0 ${width} ${color}`);

export const icon = stylex.create({
  sm: { flexShrink: 0, height: size.iconSm, width: size.iconSm },
  md: { flexShrink: 0, height: size.iconMd, width: size.iconMd },
});
