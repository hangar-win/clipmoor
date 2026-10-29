import type { ComponentPropsWithoutRef, ReactNode } from 'react';

import * as stylex from '@stylexjs/stylex';
import { TriangleAlert } from 'lucide-react';

import { icon } from '@/lib/stylex-utils';
import { border, fontSize, lineHeight, space } from '@/lib/constants.stylex';
import { ink, radius } from '@/tokens.stylex';

export type NoticeVariant = 'info' | 'warning';

type NoticeProps = Omit<ComponentPropsWithoutRef<'div'>, 'className' | 'style'> & { variant?: NoticeVariant; action?: ReactNode; style?: stylex.StyleXStyles };

export function Notice({ variant = 'info', action, style, children, ...props }: NoticeProps) {
  return (
    <div {...props} {...stylex.props(styles.notice, style)}>
      {variant === 'warning' && <TriangleAlert aria-hidden {...stylex.props(icon.sm, styles.icon)} />}
      <span {...stylex.props(styles.text)}>{children}</span>
      {action}
    </div>
  );
}

const styles = stylex.create({
  notice: {
    alignItems: 'center',
    borderColor: ink.frame,
    borderRadius: radius.control,
    borderStyle: 'solid',
    borderWidth: border.stroke,
    color: ink.text,
    display: 'flex',
    fontSize: fontSize.control,
    gap: space.s8,
    lineHeight: lineHeight.control,
    paddingBlock: space.s8,
    paddingInline: space.s12,
    textWrap: 'pretty',
  },
  icon: {
    alignSelf: 'flex-start',
    marginTop: space.s2,
  },
  text: {
    flexGrow: 1,
    minWidth: 0,
  },
});
