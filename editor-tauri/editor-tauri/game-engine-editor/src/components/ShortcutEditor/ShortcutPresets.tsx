/**
 * 快捷键预设方案
 */

import React from 'react';
import { ShortcutPreset } from '../../types/shortcuts';
import './ShortcutPresets.css';

interface ShortcutPresetsProps {
  onSelect: (preset: ShortcutPreset) => void;
  onClose: () => void;
}

export function ShortcutPresets({ onSelect, onClose }: ShortcutPresetsProps) {
  const presets: Array<{
    value: ShortcutPreset;
    name: string;
    description: string;
    features: string[];
  }> = [
    {
      value: 'default',
      name: '默认',
      description: '编辑器默认快捷键方案',
      features: ['符合人体工程学', '常用功能易用', '适合新用户'],
    },
    {
      value: 'vscode',
      name: 'VS Code',
      description: '类似 VS Code 的快捷键',
      features: ['Ctrl+P 快速打开', 'Ctrl+Shift+P 命令面板', '熟悉的编辑快捷键'],
    },
    {
      value: 'unity',
      name: 'Unity',
      description: '类似 Unity Editor 的快捷键',
      features: ['Q/W/E/R 工具切换', 'F 聚焦选中', '熟悉的 3D 操作'],
    },
    {
      value: 'unreal',
      name: 'Unreal',
      description: '类似 Unreal Editor 的快捷键',
      features: ['专业游戏开发工具', '高效的工作流', '行业标准'],
    },
    {
      value: 'blender',
      name: 'Blender',
      description: '类似 Blender 的快捷键',
      features: ['快捷键为中心', '高效建模', '专业 3D 工作流'],
    },
  ];

  return (
    <div className="presets-overlay" onClick={onClose}>
      <div className="presets-dialog" onClick={e => e.stopPropagation()}>
        <div className="presets-header">
          <h2>选择快捷键预设</h2>
          <button className="close-btn" onClick={onClose}>
            ×
          </button>
        </div>

        <div className="presets-body">
          {presets.map(preset => (
            <PresetCard
              key={preset.value}
              preset={preset}
              onSelect={() => {
                onSelect(preset.value);
                onClose();
              }}
            />
          ))}
        </div>

        <div className="presets-footer">
          <p className="presets-note">
            应用预设将覆盖当前所有自定义快捷键，建议先导出当前配置作为备份
          </p>
        </div>
      </div>
    </div>
  );
}

interface PresetCardProps {
  preset: {
    value: ShortcutPreset;
    name: string;
    description: string;
    features: string[];
  };
  onSelect: () => void;
}

function PresetCard({ preset, onSelect }: PresetCardProps) {
  return (
    <div className="preset-card">
      <div className="preset-info">
        <h3 className="preset-name">{preset.name}</h3>
        <p className="preset-description">{preset.description}</p>
        <ul className="preset-features">
          {preset.features.map((feature, index) => (
            <li key={index}>{feature}</li>
          ))}
        </ul>
      </div>
      <button className="preset-select-btn" onClick={onSelect}>
        选择
      </button>
    </div>
  );
}
