import React, { useState, useCallback } from 'react';
import { X, Upload, File, CheckCircle, XCircle, FolderOpen } from 'lucide-react';
import { assetApi } from './utils';

interface ImportDialogProps {
  isOpen: boolean;
  onClose: () => void;
  currentPath: string;
  onComplete?: (importedFiles: string[]) => void;
}

interface FileToImport {
  file: File;
  status: 'pending' | 'importing' | 'success' | 'error';
  error?: string;
}

export function ImportDialog({
  isOpen,
  onClose,
  currentPath,
  onComplete,
}: ImportDialogProps) {
  const [files, setFiles] = useState<FileToImport[]>([]);
  const [isDragging, setIsDragging] = useState(false);
  const [importOptions, setImportOptions] = useState({
    compressTextures: true,
    generateThumbnails: true,
  });

  const handleDragOver = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(true);
  }, []);

  const handleDragLeave = useCallback((e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);
  }, []);

  const handleDrop = useCallback(async (e: React.DragEvent) => {
    e.preventDefault();
    setIsDragging(false);

    const droppedFiles = Array.from(e.dataTransfer.files);
    const newFiles: FileToImport[] = droppedFiles.map((file) => ({
      file,
      status: 'pending',
    }));

    setFiles((prev) => [...prev, ...newFiles]);
  }, []);

  const handleFileSelect = useCallback((e: React.ChangeEvent<HTMLInputElement>) => {
    const selectedFiles = Array.from(e.target.files || []);
    const newFiles: FileToImport[] = selectedFiles.map((file) => ({
      file,
      status: 'pending',
    }));

    setFiles((prev) => [...prev, ...newFiles]);
  }, []);

  const removeFile = useCallback((index: number) => {
    setFiles((prev) => prev.filter((_, i) => i !== index));
  }, []);

  const handleImport = async () => {
    if (files.length === 0) return;

    // Update status to importing
    setFiles((prev) =>
      prev.map((f) => ({
        ...f,
        status: 'importing',
      }))
    );

    const importedPaths: string[] = [];

    // Import files one by one
    for (let i = 0; i < files.length; i++) {
      const fileToImport = files[i];
      try {
        // In a real implementation, we would upload the file to Tauri
        // For now, we'll simulate the import
        const result = {
          source: (fileToImport.file as any).path || fileToImport.file.name,
          destination: `${currentPath}/${fileToImport.file.name}`,
          success: true,
          error: undefined,
        };

        if (result.success) {
          setFiles((prev) =>
            prev.map((f, idx) =>
              idx === i
                ? { ...f, status: 'success' }
                : f
            )
          );
          importedPaths.push(result.destination);
        } else {
          setFiles((prev) =>
            prev.map((f, idx) =>
              idx === i
                ? { ...f, status: 'error', error: result.error }
                : f
            )
          );
        }
      } catch (error) {
        setFiles((prev) =>
          prev.map((f, idx) =>
            idx === i
              ? { ...f, status: 'error', error: String(error) }
              : f
          )
        );
      }
    }

    if (importedPaths.length > 0 && onComplete) {
      onComplete(importedPaths);
    }

    // Close dialog after a short delay
    setTimeout(() => {
      onClose();
      setFiles([]);
    }, 1500);
  };

  if (!isOpen) return null;

  const hasPendingFiles = files.some((f) => f.status === 'pending');
  const allCompleted = files.length > 0 && files.every((f) => f.status === 'success' || f.status === 'error');

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-slate-800 border border-slate-700 rounded-lg shadow-xl w-[600px] max-h-[80vh] flex flex-col">
        {/* Header */}
        <div className="flex items-center justify-between px-6 py-4 border-b border-slate-700">
          <div className="flex items-center gap-3">
            <Upload className="w-5 h-5 text-blue-400" />
            <h2 className="text-lg font-semibold">Import Assets</h2>
          </div>
          <button
            onClick={onClose}
            className="p-1 hover:bg-slate-700 rounded transition-colors"
            disabled={!allCompleted}
          >
            <X className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6 space-y-4">
          {/* Drop Zone */}
          <div
            className={`border-2 border-dashed rounded-lg p-8 text-center transition-colors ${
              isDragging
                ? 'border-blue-500 bg-blue-500/10'
                : 'border-slate-600 hover:border-slate-500'
            }`}
            onDragOver={handleDragOver}
            onDragLeave={handleDragLeave}
            onDrop={handleDrop}
          >
            <Upload className="w-12 h-12 mx-auto mb-4 text-slate-400" />
            <p className="text-slate-300 mb-2">
              Drag and drop files here, or click to browse
            </p>
            <label className="inline-flex items-center gap-2 px-4 py-2 bg-blue-600 hover:bg-blue-700 rounded-lg cursor-pointer transition-colors">
              <FolderOpen className="w-4 h-4" />
              <span>Browse Files</span>
              <input
                type="file"
                multiple
                className="hidden"
                onChange={handleFileSelect}
              />
            </label>
          </div>

          {/* File List */}
          {files.length > 0 && (
            <div className="space-y-2">
              <h3 className="text-sm font-semibold text-slate-400">Files to Import</h3>
              {files.map((fileToImport, index) => (
                <div
                  key={index}
                  className="flex items-center gap-3 p-3 bg-slate-900 rounded-lg"
                >
                  <File className="w-5 h-5 text-slate-400" />
                  <div className="flex-1 min-w-0">
                    <div className="text-sm text-slate-200 truncate">
                      {fileToImport.file.name}
                    </div>
                    <div className="text-xs text-slate-500">
                      {(fileToImport.file.size / 1024).toFixed(2)} KB
                    </div>
                  </div>

                  {fileToImport.status === 'pending' && (
                    <button
                      onClick={() => removeFile(index)}
                      className="p-1 hover:bg-slate-700 rounded"
                    >
                      <X className="w-4 h-4 text-slate-400" />
                    </button>
                  )}

                  {fileToImport.status === 'importing' && (
                    <div className="text-sm text-blue-400">Importing...</div>
                  )}

                  {fileToImport.status === 'success' && (
                    <CheckCircle className="w-5 h-5 text-green-500" />
                  )}

                  {fileToImport.status === 'error' && (
                    <div className="relative group">
                      <XCircle className="w-5 h-5 text-red-500" />
                      {fileToImport.error && (
                        <div className="absolute bottom-full left-0 mb-2 px-2 py-1 bg-red-600 text-white text-xs rounded whitespace-nowrap opacity-0 group-hover:opacity-100 transition-opacity">
                          {fileToImport.error}
                        </div>
                      )}
                    </div>
                  )}
                </div>
              ))}
            </div>
          )}

          {/* Import Options */}
          <div className="space-y-2">
            <h3 className="text-sm font-semibold text-slate-400">Import Options</h3>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={importOptions.compressTextures}
                onChange={(e) =>
                  setImportOptions((prev) => ({
                    ...prev,
                    compressTextures: e.target.checked,
                  }))
                }
                className="w-4 h-4 rounded border-slate-600"
              />
              <span className="text-sm text-slate-200">Compress textures</span>
            </label>
            <label className="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={importOptions.generateThumbnails}
                onChange={(e) =>
                  setImportOptions((prev) => ({
                    ...prev,
                    generateThumbnails: e.target.checked,
                  }))
                }
                className="w-4 h-4 rounded border-slate-600"
              />
              <span className="text-sm text-slate-200">Generate thumbnails</span>
            </label>
          </div>

          {/* Destination */}
          <div>
            <h3 className="text-sm font-semibold text-slate-400 mb-2">Destination</h3>
            <div className="p-3 bg-slate-900 rounded-lg text-sm text-slate-200 font-mono">
              {currentPath}
            </div>
          </div>
        </div>

        {/* Footer */}
        <div className="flex items-center justify-end gap-3 px-6 py-4 border-t border-slate-700">
          <button
            onClick={onClose}
            className="px-4 py-2 text-slate-300 hover:bg-slate-700 rounded-lg transition-colors"
            disabled={!allCompleted && files.length > 0}
          >
            Cancel
          </button>
          <button
            onClick={handleImport}
            disabled={files.length === 0 || !hasPendingFiles}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:bg-slate-600 disabled:text-slate-400 rounded-lg transition-colors"
          >
            Import {files.length > 0 ? `(${files.length})` : ''}
          </button>
        </div>
      </div>
    </div>
  );
}
