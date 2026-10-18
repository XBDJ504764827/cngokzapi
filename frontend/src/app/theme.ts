import type { GlobalThemeOverrides } from 'naive-ui'

// 主色与旧原型一致：白底 + 单一墨绿。细边框为主，阴影只用于 hover 的轻微浮起
// （沿用原型 tokens：radius 6/10/16、shadow-sm/md），保持"克制干净、不做科技感"。
export const themeOverrides: GlobalThemeOverrides = {
  common: {
    primaryColor: '#2F6F4F',
    primaryColorHover: '#255A40',
    primaryColorPressed: '#1E4A35',
    primaryColorSuppl: '#255A40',
    successColor: '#2F6F4F',
    successColorHover: '#255A40',
    successColorPressed: '#1E4A35',
    borderRadius: '10px',
    borderRadiusSmall: '6px',
    fontFamily:
      "'Inter', -apple-system, BlinkMacSystemFont, 'PingFang SC', 'Microsoft YaHei', sans-serif",
    fontSize: '14px',
    textColor1: '#1C1C1A',
    textColor2: '#5B5A56',
    textColor3: '#8E8C86',
    borderColor: '#E6E4DF',
    dividerColor: '#E9E7E2',
    bodyColor: '#FAFAF9',
    cardColor: '#FFFFFF',
    hoverColor: '#F3F2EF',
    tableHeaderColor: '#F7F6F3',
    tableColorHover: '#F8F7F4',
    boxShadow1: '0 1px 2px rgba(28, 28, 26, 0.04)',
    boxShadow2: '0 4px 16px rgba(28, 28, 26, 0.06)',
    boxShadow3: '0 8px 28px rgba(28, 28, 26, 0.08)',
  },
  Card: {
    borderRadius: '14px',
    color: '#FFFFFF',
    borderColor: '#E6E4DF',
    titleFontSizeMedium: '15px',
    titleFontWeight: '600',
    titleFontWeightSmall: '600',
    paddingMedium: '20px 22px',
    paddingSmall: '14px 16px',
  },
  Alert: {
    borderRadius: '10px',
  },
  DataTable: {
    borderRadius: '12px',
    thColor: '#F7F6F3',
    thTextColor: '#5B5A56',
    thFontWeight: '600',
    tdColorHover: '#F8F7F4',
    borderColor: '#EDEBE6',
    thPaddingMedium: '10px 14px',
    tdPaddingMedium: '11px 14px',
  },
  Tag: {
    borderRadius: '999px',
    heightSmall: '22px',
    fontWeightStrong: '600',
  },
  Button: {
    borderRadiusMedium: '10px',
    fontWeight: '600',
    fontWeightStrong: '600',
  },
  Input: {
    borderRadius: '10px',
  },
  Pagination: {
    itemBorderRadius: '8px',
  },
  Statistic: {
    labelFontSize: '13px',
    labelTextColor: '#8E8C86',
    valueFontSize: '28px',
    valueFontWeight: '700',
    valueTextColor: '#2F6F4F',
  },
}
