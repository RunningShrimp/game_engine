/**
 * Material Editor Example
 * 示例页面，展示如何集成和使用材质编辑器
 */

import React, { useState } from 'react';
import { MaterialEditor } from './MaterialEditor';
import { Material } from '../../types/material';
import './MaterialEditorExample.css';

export const MaterialEditorExample: React.FC = () => {
  const [material, setMaterial] = useState<Material>({
    id: 'example_material',
    name: 'Example Material',
    nodes: [],
    connections: [],
    previewMesh: 'sphere',
  });

  const handleMaterialChange = (updatedMaterial: Material) => {
    setMaterial(updatedMaterial);
    console.log('Material updated:', updatedMaterial);
  };

  return (
    <div className="material-editor-example">
      <div className="example-header">
        <h1>Material Editor - Example</h1>
        <p>节点式材质编辑器演示</p>
      </div>

      <div className="example-content">
        <MaterialEditor
          initialMaterial={material}
          onMaterialChange={handleMaterialChange}
        />
      </div>

      <div className="example-footer">
        <div className="example-info">
          <h3>使用说明：</h3>
          <ul>
            <li>从左侧面板点击添加节点</li>
            <li>拖拽节点的输出端口到另一个节点的输入端口以创建连接</li>
            <li>在节点中直接编辑参数</li>
            <li>在右侧预览面板查看实时效果</li>
            <li>使用Ctrl+S保存材质</li>
            <li>使用Ctrl+C/V复制粘贴节点</li>
            <li>使用Delete删除选中的节点</li>
          </ul>
        </div>
      </div>
    </div>
  );
};

export default MaterialEditorExample;
