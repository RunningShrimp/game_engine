/**
 * Material Manager Component
 * Handles material saving, loading, and preset management
 */

import React, { useState, useEffect } from 'react';
import { Material, MaterialPreset, MaterialExport } from '../../types/material';
import { MaterialEditor } from './MaterialEditor';
import './MaterialManager.css';

interface MaterialManagerProps {
  material: Material;
  onLoad: (material: Material) => void;
  onClose: () => void;
}

// Material presets
const MATERIAL_PRESETS: MaterialPreset[] = [
  {
    name: 'Basic',
    description: 'Basic diffuse material',
    material: {
      id: 'preset_basic',
      name: 'Basic',
      nodes: [
        {
          id: 'pbr_1',
          type: 'pbr_master' as any,
          position: { x: 100, y: 100 },
          inputs: [
            { id: 'baseColor', name: 'Base Color', dataType: 'color' as any, connectedTo: null, nodeId: '', isInput: true },
            { id: 'metallic', name: 'Metallic', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: true },
            { id: 'roughness', name: 'Roughness', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: true },
            { id: 'normal', name: 'Normal', dataType: 'vector3' as any, connectedTo: null, nodeId: '', isInput: true },
            { id: 'ao', name: 'AO', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: true },
          ],
          outputs: [],
          parameters: [
            { id: 'baseColor', name: 'Base Color', dataType: 'color' as any, value: [1, 1, 1, 1] },
            { id: 'metallic', name: 'Metallic', dataType: 'float' as any, value: 0, min: 0, max: 1, step: 0.01 },
            { id: 'roughness', name: 'Roughness', dataType: 'float' as any, value: 0.5, min: 0, max: 1, step: 0.01 },
            { id: 'ao', name: 'AO', dataType: 'float' as any, value: 1, min: 0, max: 1, step: 0.01 },
          ],
        },
      ],
      connections: [],
      previewMesh: 'sphere',
    },
  },
  {
    name: 'Metal',
    description: 'Metallic material with roughness',
    material: {
      id: 'preset_metal',
      name: 'Metal',
      nodes: [
        {
          id: 'pbr_1',
          type: 'pbr_master' as any,
          position: { x: 100, y: 100 },
          inputs: [
            { id: 'baseColor', name: 'Base Color', dataType: 'color' as any, connectedTo: null, nodeId: '', isInput: true },
            { id: 'metallic', name: 'Metallic', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: true },
            { id: 'roughness', name: 'Roughness', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: true },
            { id: 'normal', name: 'Normal', dataType: 'vector3' as any, connectedTo: null, nodeId: '', isInput: true },
            { id: 'ao', name: 'AO', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: true },
          ],
          outputs: [],
          parameters: [
            { id: 'baseColor', name: 'Base Color', dataType: 'color' as any, value: [0.8, 0.8, 0.8, 1] },
            { id: 'metallic', name: 'Metallic', dataType: 'float' as any, value: 1, min: 0, max: 1, step: 0.01 },
            { id: 'roughness', name: 'Roughness', dataType: 'float' as any, value: 0.3, min: 0, max: 1, step: 0.01 },
            { id: 'ao', name: 'AO', dataType: 'float' as any, value: 1, min: 0, max: 1, step: 0.01 },
          ],
        },
      ],
      connections: [],
      previewMesh: 'sphere',
    },
  },
  {
    name: 'Plastic',
    description: 'Plastic material with low roughness',
    material: {
      id: 'preset_plastic',
      name: 'Plastic',
      nodes: [
        {
          id: 'pbr_1',
          type: 'pbr_master' as any,
          position: { x: 100, y: 100 },
          inputs: [
            { id: 'baseColor', name: 'Base Color', dataType: 'color' as any, connectedTo: null, nodeId: '', isInput: true },
            { id: 'metallic', name: 'Metallic', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: true },
            { id: 'roughness', name: 'Roughness', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: true },
            { id: 'normal', name: 'Normal', dataType: 'vector3' as any, connectedTo: null, nodeId: '', isInput: true },
            { id: 'ao', name: 'AO', dataType: 'float' as any, connectedTo: null, nodeId: '', isInput: true },
          ],
          outputs: [],
          parameters: [
            { id: 'baseColor', name: 'Base Color', dataType: 'color' as any, value: [0.2, 0.5, 0.8, 1] },
            { id: 'metallic', name: 'Metallic', dataType: 'float' as any, value: 0, min: 0, max: 1, step: 0.01 },
            { id: 'roughness', name: 'Roughness', dataType: 'float' as any, value: 0.2, min: 0, max: 1, step: 0.01 },
            { id: 'ao', name: 'AO', dataType: 'float' as any, value: 1, min: 0, max: 1, step: 0.01 },
          ],
        },
      ],
      connections: [],
      previewMesh: 'sphere',
    },
  },
];

export const MaterialManager: React.FC<MaterialManagerProps> = ({
  material,
  onLoad,
  onClose,
}) => {
  const [savedMaterials, setSavedMaterials] = useState<Material[]>([]);
  const [activeTab, setActiveTab] = useState<'saved' | 'presets'>('saved');

  // Load saved materials from localStorage
  useEffect(() => {
    const saved = localStorage.getItem('saved_materials');
    if (saved) {
      try {
        setSavedMaterials(JSON.parse(saved));
      } catch (e) {
        console.error('Failed to load saved materials:', e);
      }
    }
  }, []);

  // Save current material
  const handleSave = () => {
    const updated = [...savedMaterials, material];
    setSavedMaterials(updated);
    localStorage.setItem('saved_materials', JSON.stringify(updated));

    // Save to file
    saveMaterialToFile(material);
  };

  // Save material to file
  const saveMaterialToFile = (mat: Material) => {
    const exportData: MaterialExport = {
      version: '1.0',
      material: mat,
      metadata: {
        exportedAt: new Date().toISOString(),
        editorVersion: '1.0.0',
      },
    };

    const blob = new Blob([JSON.stringify(exportData, null, 2)], {
      type: 'application/json',
    });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `${mat.name.replace(/\s+/g, '_').toLowerCase()}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  // Export material
  const handleExport = () => {
    saveMaterialToFile(material);
  };

  // Import material from file
  const handleImport = () => {
    const input = document.createElement('input');
    input.type = 'file';
    input.accept = '.json';
    input.onchange = async (e) => {
      const file = (e.target as HTMLInputElement).files?.[0];
      if (!file) return;

      try {
        const text = await file.text();
        const data: MaterialExport = JSON.parse(text);
        onLoad(data.material);
      } catch (e) {
        console.error('Failed to import material:', e);
        alert('Failed to import material file');
      }
    };
    input.click();
  };

  // Load material from list
  const handleLoadMaterial = (mat: Material) => {
    onLoad(mat);
  };

  // Delete saved material
  const handleDeleteMaterial = (materialId: string) => {
    const updated = savedMaterials.filter(m => m.id !== materialId);
    setSavedMaterials(updated);
    localStorage.setItem('saved_materials', JSON.stringify(updated));
  };

  return (
    <div className="material-manager-overlay" onClick={onClose}>
      <div className="material-manager" onClick={(e) => e.stopPropagation()}>
        <div className="manager-header">
          <h2>Material Manager</h2>
          <button className="close-button" onClick={onClose}>
            ×
          </button>
        </div>

        <div className="manager-tabs">
          <button
            className={activeTab === 'saved' ? 'active' : ''}
            onClick={() => setActiveTab('saved')}
          >
            Saved Materials
          </button>
          <button
            className={activeTab === 'presets' ? 'active' : ''}
            onClick={() => setActiveTab('presets')}
          >
            Presets
          </button>
        </div>

        <div className="manager-content">
          <div className="manager-actions">
            <button className="action-button save" onClick={handleSave}>
              💾 Save Current
            </button>
            <button className="action-button export" onClick={handleExport}>
              📤 Export
            </button>
            <button className="action-button import" onClick={handleImport}>
              📥 Import
            </button>
          </div>

          {activeTab === 'saved' && (
            <div className="material-list">
              <h3>Saved Materials</h3>
              {savedMaterials.length === 0 ? (
                <p className="empty-message">No saved materials yet</p>
              ) : (
                savedMaterials.map(mat => (
                  <div key={mat.id} className="material-item">
                    <div className="material-info">
                      <h4>{mat.name}</h4>
                      <p>{mat.nodes.length} nodes</p>
                    </div>
                    <div className="material-actions">
                      <button
                        className="icon-button"
                        onClick={() => handleLoadMaterial(mat)}
                        title="Load"
                      >
                        📂
                      </button>
                      <button
                        className="icon-button"
                        onClick={() => handleDeleteMaterial(mat.id)}
                        title="Delete"
                      >
                        🗑️
                      </button>
                    </div>
                  </div>
                ))
              )}
            </div>
          )}

          {activeTab === 'presets' && (
            <div className="material-list">
              <h3>Material Presets</h3>
              {MATERIAL_PRESETS.map(preset => (
                <div key={preset.material.id} className="material-item">
                  <div className="material-info">
                    <h4>{preset.name}</h4>
                    <p>{preset.description}</p>
                  </div>
                  <div className="material-actions">
                    <button
                      className="icon-button"
                      onClick={() => handleLoadMaterial(preset.material)}
                      title="Load"
                    >
                      📂
                    </button>
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>
      </div>
    </div>
  );
};

export default MaterialManager;
