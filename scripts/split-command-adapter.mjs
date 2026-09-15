import fs from 'fs';
import path from 'path';
import { execSync } from 'child_process';

const commandAdapterPath = 'frontend/src/lib/api/commandAdapter.ts';
const fullOriginal = execSync('git show HEAD:frontend/src/lib/api/commandAdapter.ts', { encoding: 'utf8' });

processSplitting(fullOriginal);

function processSplitting(originalContent) {
  const lines = originalContent.split('\n');
  const funcPositions = [];

  lines.forEach((line, i) => {
    const match = line.match(/^(?:export\s+)?(?:async\s+)?function\s+([a-zA-Z0-9_]+)/);
    if (match) {
      const name = match[1];
      let startLine = i;
      while (startLine > 0 && (lines[startLine - 1].trim().startsWith('*') || lines[startLine - 1].trim().startsWith('/**') || lines[startLine - 1].trim().startsWith('//'))) {
        startLine--;
      }
      funcPositions.push({ name, startLine, funcLine: i });
    }
  });

  funcPositions.sort((a, b) => a.startLine - b.startLine);

  const funcBlocks = {};
  for (let i = 0; i < funcPositions.length; i++) {
    const current = funcPositions[i];
    const next = funcPositions[i + 1];
    const endLine = next ? next.startLine : lines.length;
    let blockCode = lines.slice(current.startLine, endLine).join('\n').trim();
    funcBlocks[current.name] = blockCode;
  }

  const mockConstants = `export const MOCK_DESIGNS = [
  {
    id: 1,
    filename: "rose-border-01.pes",
    designer: "Mock Designer",
    source: "Mock Source",
    tags: ["Flowers", "Borders"],
    hoop: "Hoop A",
    rating: 4,
    is_stitched: false,
    image_tags_verified: true,
    stitching_tags_verified: true,
  },
  {
    id: 2,
    filename: "holiday-tree.vp3",
    designer: "Mock Studio",
    source: "Imported",
    tags: ["Christmas"],
    hoop: "Hoop B",
    rating: 3,
    is_stitched: true,
    image_tags_verified: false,
    stitching_tags_verified: false,
  },
  {
    id: 3,
    filename: "monogram-a.dst",
    designer: "Mock Designer",
    source: "Purchased",
    tags: ["Words and Letters"],
    hoop: null,
    rating: null,
    is_stitched: false,
    image_tags_verified: true,
    stitching_tags_verified: true,
  },
];

export const TAG_SEED = [
  "Line Outline",
  "Satin Stitch",
  "Applique",
  "Food",
  "Nautical",
  "Words and Letters",
  "Floral",
  "Butterflies and Insects",
];

export const MOCK_HOOPS = [
  { id: 1, name: "Hoop A" },
  { id: 2, name: "Hoop B" },
  { id: 3, name: "Hoop C" },
];`;

  const domainMap = {
    designs: {
      filename: 'designsAdapter.ts',
      extras: mockConstants,
      funcs: [
        'normalizeBrowseItem',
        'getBrowseDesigns', 'getDesignIds', 'getDesignDetail', 'getDesignImageDataUrl', 'updateDesignMetadata',
        'setDesignRating', 'setDesignStitched', 'setDesignVerification', 'setDesignTags', 'removeDesignTag',
        'bulkDeleteDesigns', 'openDesignInEditor', 'openDesignInExplorer', 'renderDesign3dPreview', 'reparseDesignFile',
        'bulkVerifyDesigns', 'getBrowseDesignPreviews'
      ]
    },
    projects: {
      filename: 'projectsAdapter.ts',
      funcs: [
        'addDesignToProject', 'removeDesignFromProject', 'getBrowseProjects', 'getProjectsList', 'createProject',
        'getProjectDetail', 'updateProject', 'deleteProject', 'removeDesignFromProjectDetail', 'getProjectPrintView',
        'bulkAddDesignsToProject'
      ]
    },
    import: {
      filename: 'importAdapter.ts',
      funcs: [
        'previewImportFromRoots', 'previewImportFromRoot', 'browseImportFolder', 'precheckImportWire',
        'runPrecheckAction', 'requestStopBulkImport'
      ]
    },
    tags: {
      filename: 'tagsAdapter.ts',
      funcs: [
        'getBrowseTags', 'bulkSetTagsForDesigns', 'listTags', 'createTag', 'setTagGroup', 'updateTag', 'deleteTag'
      ]
    },
    batchOperations: {
      filename: 'batchOperationsAdapter.ts',
      funcs: [
        'getBatchOperationsViewModel', 'countTaggingCandidates', 'browseTaggingFolder', 'buildUnifiedBackfillWireRequest',
        'runUnifiedBackfill', 'stopUnifiedBackfill', 'getBackfillLogEntries', 'runStitchingBackfill', 'runMaintenanceBackfill',
        'countMissingPreviews'
      ]
    },
    backup: {
      filename: 'backupAdapter.ts',
      funcs: [
        'getBackupViewModel', 'saveBackupSettings', 'browseBackupFolder', 'runDatabaseBackup', 'requestCancelBackup',
        'runDesignsBackup', 'runBothBackups', 'browseRestoreFile', 'normalizeRestoreDatabase',
        'restoreDatabase', 'restoreDesignsIncremental', 'restoreBoth', 'detectDesignFilesAbsentFromDatabase',
        'importUnmatchedDesignFiles', 'requestCancelRestore'
      ]
    },
    orphans: {
      filename: 'orphansAdapter.ts',
      funcs: [
        'scanOrphans', 'getOrphansPage', 'deleteOrphans', 'deleteAllOrphans', 'browseOrphanPath'
      ]
    },
    admin: {
      filename: 'adminAdapter.ts',
      funcs: [
        'getDbStats', 'compactDatabase', 'listDesigners', 'createDesigner', 'updateDesigner', 'deleteDesigner',
        'listSources', 'createSource', 'updateSource', 'deleteSource', 'listHoops', 'createHoop', 'updateHoop', 'deleteHoop'
      ]
    },
    settings: {
      filename: 'settingsAdapter.ts',
      funcs: [
        'getAboutDocuments', 'getAboutDocument', 'getSettingsViewModel', 'saveSettings', 'listGeminiModels',
        'testGeminiModel', 'saveImportLastBrowseFolder', 'browseSettingsDataRoot', 'getDatabaseStatus',
        'detectRelocatedDataRoot', 'validateDatabasePath', 'seedDatabaseToDataRoot', 'getGoogleApiKey',
        'setGoogleApiKey', 'checkInitialSetup', 'completeInitialSetup', 'getAppStatus', 'getConfiguredDataRoot',
        'setConfiguredDataRoot', 'configureFreshDataRoot', 'browseDataRootFolder', 'restartApplication',
        'startCatalogueStorageMigration', 'cancelCatalogueStorageMigration', 'listenCatalogueStorageMigrationProgress'
      ]
    }
  };

  // Check mapping coverage
  const allMappedFuncs = [];
  for (const domain of Object.values(domainMap)) {
    allMappedFuncs.push(...domain.funcs);
  }
  console.log(`Total domain funcs mapped: ${allMappedFuncs.length}`);

  const originalFuncNames = funcPositions.map(f => f.name).filter(n => n !== 'invokeLoose');
  const unmapped = originalFuncNames.filter(f => !allMappedFuncs.includes(f));
  if (unmapped.length) {
    console.warn('Unmapped functions:', unmapped);
  }

  // Extract all imported types from ipc.ts
  const allIpcTypes = [
    'AdapterBackfillLogEntriesResponse', 'AdapterBackupViewModelResponse', 'AdapterBrowseBackupFolderResponse',
    'AdapterBrowseDesignsPageResponse', 'AdapterBrowseImportFolderResponse', 'AdapterBrowseOrphanPathResponse',
    'AdapterDeleteOrphansResponse', 'AdapterImportPrecheckActionResponse', 'AdapterImportPrecheckResponse',
    'AdapterImportPreviewResponse', 'AdapterOrphansPageResponse', 'AdapterPersistedItemResponse',
    'AdapterPersistedResponse', 'AdapterProjectDesignMutationResponse', 'AdapterProjectDetailResponse',
    'AdapterProjectListResponse', 'AdapterProjectMutationResponse', 'AdapterReparseDesignResponse',
    'AdapterRunBothBackupsResponse', 'AdapterSaveBackupSettingsResponse', 'AdapterScanOrphansResponse',
    'AdapterStopBulkImportResponse', 'AdapterStopUnifiedBackfillResponse', 'AdapterBatchOperationsViewModelResponse',
    'AdapterTaggingCandidateCountResponse', 'BrowseTaggingFolderResult', 'TaggingScopeCounts',
    'AdapterAppStatusResponse', 'AdapterBrowseDataRootResponse', 'AdapterCompactResponse',
    'AdapterConfigureDataRootResponse', 'AdapterDbStatsResponse', 'AdapterGoogleApiKeyResponse',
    'AdapterItemResponse', 'AdapterListResponse', 'AdapterMutationResponse', 'AdapterSaveSettingsResponse',
    'AdapterSettingsViewModelResponse', 'AdminEntitySummary', 'AdminHoopSummary', 'AdminTagSummary',
    'AppStatus', 'BackupViewModel', 'BulkImportPreview', 'BrowseImportFolderResult', 'BrowseDesignPreview',
    'BrowseDesignSummaryWire', 'BrowseTagOption', 'CancelBackupResult', 'CancelRestoreResult',
    'CompactResult', 'ConfigureDataRootResult', 'DatabaseBackupResult', 'DatabaseStatus',
    'DatabaseValidation', 'DetectUnmatchedFilesResult', 'DetectedDataRoot', 'DbStats',
    'DesignCommandResult', 'DesignDetail', 'DesignDetailWire', 'DesignsBackupResult',
    'DesignImageData', 'ImportPrecheckActionResult', 'ImportPrecheckResult', 'GeminiModelTestResult',
    'ProjectDetailView', 'ProjectMutationResult', 'ProjectListItem', 'ProjectSummary',
    'RemoveProjectDesignResult', 'ReparseDesignResultWire', 'RunStitchingBackfillOptions',
    'SaveBackupSettingsRequest', 'SaveSettingsRequest', 'SearchPayload', 'SettingsViewModel',
    'BatchOperationsViewModel', 'UnifiedBackfillActionsWire', 'UnifiedBackfillRequest',
    'UnifiedBackfillResult', 'UnifiedBackfillWireRequest', 'UpdateDesignMetadataRequest',
    'StorageMigrationProgress', 'StorageMigrationSummary', 'ImportUnmatchedFilesResult',
    'RestoreBothResult', 'RestoreDatabaseResult', 'RestoreDesignsResult', 'BrowseRestoreFileResponse'
  ];

  // Specific helpers from ../types/ipc
  const ipcHelpers = ['mapDesignDetailFromWire', 'mapReparseDesignFromWire'];

  // Write domain files
  for (const [domainKey, domainDef] of Object.entries(domainMap)) {
    const targetPath = path.join('frontend/src/lib/api', domainDef.filename);

    let body = '';
    if (domainDef.extras) {
      body += domainDef.extras + '\n\n';
    }

    for (const funcName of domainDef.funcs) {
      let block = funcBlocks[funcName];
      if (!block) {
        console.error('Missing block for', funcName);
      } else {
        body += block + '\n\n';
      }
    }

    // Determine which types are used in body
    const usedTypes = allIpcTypes.filter(t => new RegExp(`\\b${t}\\b`).test(body));
    const usedHelpers = ipcHelpers.filter(h => new RegExp(`\\b${h}\\b`).test(body));

    const usesLooseRecord = /\bLooseRecord\b/.test(body);
    let header = usesLooseRecord
      ? `import { invokeLoose, type LooseRecord } from "./ipcClient";\n`
      : `import { invokeLoose } from "./ipcClient";\n`;
    if (usedTypes.length) {
      header += `import type {\n  ${usedTypes.join(',\n  ')},\n} from "../types/ipc";\n`;
    }
    if (usedHelpers.length) {
      header += `import { ${usedHelpers.join(', ')} } from "../types/ipc";\n`;
    }
    header += '\n';

    fs.writeFileSync(targetPath, header + body.trim() + '\n', 'utf8');
    console.log('Wrote', targetPath, 'with', domainDef.funcs.length, 'functions and', usedTypes.length, 'imported types.');
  }

  // Write new commandAdapter.ts that re-exports everything
  let reExportContent = `// Backward compatibility barrel re-exporting all domain adapters\n`;
  reExportContent += `export * from "./ipcClient";\n`;
  for (const domainDef of Object.values(domainMap)) {
    const baseName = domainDef.filename.replace('.ts', '');
    reExportContent += `export * from "./${baseName}";\n`;
  }

  fs.writeFileSync(commandAdapterPath, reExportContent, 'utf8');
  console.log('Updated', commandAdapterPath, 'to re-export all domain adapters.');
}
