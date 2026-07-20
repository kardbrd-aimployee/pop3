# Reverse Engineering Meta — Named Functions

**Total named: 1,475** | Binary: `popTB.exe` (Win32 x86)

---

## AI System (126 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0040bd30 | AI_ProcessBuildingEject | 1 |
| 0041a7d0 | AI_UpdateAllTribes | 3 |
| 0041a8b0 | AI_UpdateTribe | 1 |
| 0041b000 | AI_CalculateThreatDistance | 2 |
| 0041b1b0 | AI_UpdateShamanStatus | 0 |
| 0041b280 | AI_ValidateBuildingPlacements | 1 |
| 0041b6d0 | AI_ProcessShamanCommands | 1 |
| 0041b8d0 | AI_ExecuteBuildingPriorities | 1 |
| 0041ba20 | AI_FindFreePersonSlot | 18 |
| 0041ba40 | AI_AssessThreat | 12 |
| 0041ba60 | AI_ClearObjectCommandFlags | 43 |
| 0041ba80 | AI_SetShamanCommand | 33 |
| 0041bae0 | AI_CheckShamanSafety | 24 |
| 0041bf90 | AI_ShamanRetreat | 4 |
| 00423b10 | AI_ResetPersonsByState | 34 |
| 004242b0 | AI_IsTribeReady | 20 |
| 00427840 | AI_KillTribeUnits | 1 |
| 00443590 | AI_ProcessAttackSpell | 1 |
| 00443810 | AI_ProcessGatherAttack | 1 |
| 00443f20 | AI_ProcessQueuedSpell | 1 |
| 00444640 | AI_Cmd_PrimaryAttack | 1 |
| 00444c10 | AI_Cmd_DefendPosition | 1 |
| 00445300 | AI_Cmd_SecondaryAttack | 1 |
| 00445910 | AI_Cmd_BuildingPlacement | 1 |
| 00445a40 | AI_Cmd_SpellCasting | 1 |
| 00445d30 | AI_Cmd_ResourceGathering | 1 |
| 004461b0 | AI_Cmd_Conversion | 1 |
| 004464f0 | AI_ProcessBuildCommand | 1 |
| 00446880 | AI_ProcessConvertCommand | 1 |
| 00446be0 | AI_ProcessSpyCommand | 1 |
| 00446e20 | AI_ProcessPatrolCommand | 1 |
| 004471c0 | AI_ProcessGarrisonCommand | 1 |
| 00447490 | AI_ProcessShamanMoveCommand | 1 |
| 00447bc0 | AI_ProcessRaidCommand | 1 |
| 00448360 | AI_CommandBuildHut | 1 |
| 00448ca0 | AI_CommandAttack | 1 |
| 0044a430 | AI_Cmd_ArmyMovement | 1 |
| 0044c6c0 | AI_CommandGatherUnits | 1 |
| 0044cbd0 | AI_CommandEvacuateUnits | 1 |
| 0044cdb0 | AI_CommandTrainUnits | 1 |
| 0044d260 | AI_CommandBuildAndGuard | 1 |
| 0044ddb0 | AI_CommandDefend | 1 |
| 0047ac90 | AI_AddCommandEntry | 63 |
| 0047af10 | AI_DispatchCommandEntries | 37 |
| 004a40a0 | AI_FindNearestDefendGroup | 1 |
| 004a43e0 | AI_FindSpellTargetBuilding | 1 |
| 004b3a10 | AI_FindNearestSafeCell | 1 |
| 004b3f10 | AI_UpdateUnitCooldowns | 2 |
| 004b3f30 | AI_ValidateTargets | 2 |
| 004b3f60 | AI_CanBuildMore | 18 |
| 004b41a0 | AI_CountAvailableBuildings | 1 |
| 004b4410 | AI_IsPersonAvailable | 12 |
| 004b51c0 | AI_CountEnemyUnits | 2 |
| 004b5e10 | AI_CheckMarkerNearPosition | 2 |
| 004b6340 | AI_ScriptCmd_CheckShamanSafe | 1 |
| 004b6390 | AI_ScriptCmd_EnsureShamanSafe | 1 |
| 004b6c50 | AI_PlaceBuilding | 19 |
| 004b77c0 | AI_FindTargetsInArea | 5 |
| 004b7af0 | AI_AssignPersonTarget | 29 |
| 004b7b80 | AI_SetPersonTarget | 24 |
| 004b7ba0 | AI_FindValidDefendPosition | 3 |
| 004b7e90 | AI_GetTargetPerson | 39 |
| 004b8130 | Building_HasSlotType | 16 |
| 004b8190 | AI_EvaluateAttackThreat | 2 |
| 004b82b0 | AI_ClearCommandSlotIfComplete | 47 |
| 004b8450 | AI_DismissPersonsToCell | 10 |
| 004b86b0 | AI_ReleasePersonsToIdle | 44 |
| 004b8a90 | AI_EvaluateSpellCasting | 2 |
| 004b9770 | AI_FindBestAttackTarget | 1 |
| 004c5e50 | AI_SetTribeData | 19 |
| 004c5eb0 | AI_RunScript | 1 |
| 004c6180 | AI_ProcessScriptBlock | 6 |
| 004c6460 | AI_ExecuteScriptCommand | 2 |
| 004c8590 | AI_ProcessSubroutineCall | 2 |
| 004c8700 | AI_ProcessLoopCommand | 2 |
| 004c8860 | AI_EvaluateCondition | 4 |
| 004c8930 | AI_EvaluateComparison | 5 |
| 004c8a30 | AI_EvaluateConditionExpression | 3 |
| 004c8b50 | AI_EvaluateScriptValue | 184 |
| 004c9450 | AI_ScriptCmd_AttackWithArmy | 1 |
| 004c9760 | AI_ScriptCmd_SetAttackParams | 1 |
| 004c9950 | AI_ScriptCmd_SetTribeProperty | 1 |
| 004c99e0 | AI_ScriptCmd_SetViewTarget | 1 |
| 004c9ae0 | AI_ScriptCmd_ConfigureDefense | 1 |
| 004c9c40 | AI_ScriptCmd_CastSpellAtTarget | 1 |
| 004c9cd0 | AI_ScriptCmd_LookAtShaman | 1 |
| 004c9d90 | AI_ScriptCmd_CastSpellDirect | 1 |
| 004c9e80 | AI_ScriptCmd_CastSpellArea | 1 |
| 004c9f30 | AI_ScriptCmd_CastSpellDirectional | 1 |
| 004c9ff0 | AI_ScriptCmd_CastSpellTargeted | 1 |
| 004ca0e0 | AI_ScriptCmd_CastSpellComplex | 1 |
| 004ca210 | AI_ScriptCmd_SetGuardArea | 1 |
| 004ca2b0 | AI_ScriptCmd_SetPatrolTarget | 1 |
| 004ca3a0 | AI_ScriptCmd_SetGuardRegion | 1 |
| 004ca440 | AI_ScriptCmd_CastSpell | 1 |
| 004ca540 | AI_ScriptCmd_GetTribePersonCount | 1 |
| 004ca640 | AI_ScriptCmd_GetAttribute | 1 |
| 004ca7a0 | AI_ScriptCmd_GetAttributeCount | 1 |
| 004ca840 | AI_ScriptCmd_SendPersonToPos | 1 |
| 004caa40 | AI_ScriptCmd_SetPersonTarget | 1 |
| 004cab50 | AI_ScriptCmd_KillPersonsInArea | 1 |
| 004cabf0 | AI_ScriptCmd_GetIdlePersonCount | 1 |
| 004cac90 | AI_ScriptCmd_SetAttackTarget | 1 |
| 004cad70 | AI_ScriptCmd_DisableProperty | 1 |
| 004cae90 | AI_ScriptCmd_SetBucketUsage | 1 |
| 004caef0 | AI_ScriptCmd_EnableProperty | 1 |
| 004cb010 | AI_ScriptCmd_SetActiveSpell | 1 |
| 004cb1e0 | AI_ScriptCmd_TrainPersons | 1 |
| 004cb290 | AI_ScriptCmd_AttackWithSpell | 1 |
| 004cb360 | AI_ScriptCmd_DefendWithSpell | 1 |
| 004cb430 | AI_ScriptCmd_CountPersonsInRange | 1 |
| 004cb4d0 | AI_ScriptCmd_CountPersonsByType | 1 |
| 004cb5b0 | AI_ScriptCmd_FindTargetObject | 1 |
| 004cb890 | AI_ScriptCmd_SetPatrolPoints | 1 |
| 004cb960 | AI_ScriptCmd_SetPersonState | 1 |
| 004cba40 | AI_ScriptCmd_CountObjectsInArea | 1 |
| 004cbbb0 | AI_ScriptDefineArea | 1 |
| 004cbd00 | AI_ScriptCheckBuildingState | 1 |
| 004cbde0 | AI_ScriptSetMarkerPosition | 1 |
| 004cbe80 | AI_ScriptCountPeopleInArea | 1 |
| 004cbf90 | AI_ScriptGetTribeAttribute | 1 |
| 004cc060 | AI_ScriptCountTribeBuildings | 1 |
| 004cc120 | AI_ScriptGetTerrainHeight | 1 |
| 004cc1c0 | AI_ExecuteMultiParamCommand | 2 |
| 004cc340 | AI_ScriptSetGuardPoint | 1 |
| 004cc430 | AI_ScriptSetSpellTarget | 1 |
| 004da7c0 | AI_ScriptCmd_SetRenderDirty | 1 |
| 004ec0e0 | AI_CheckShamanBeforeBuild | 1 |

## Animation (5 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0041fea0 | Animation_FreeAllBoneData | 1 |
| 00421140 | Animation_UpdateBoneHierarchy | 2 |
| 00452530 | Animation_LoadAllData | 1 |
| 004b0ad0 | Object_SetShapeFromType | 94 |
| 004e7190 | Animation_RenderFrameSequence | 2 |

## Building (55 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 004166a0 | Building_MarkConvertVisible | 3 |
| 00426220 | Building_CalcPopGrowthRate | 3 |
| 00426380 | Building_AddTribeStat | 15 |
| 0042e230 | Building_Init | 1 |
| 0042e430 | Building_SetState | 2 |
| 0042e5f0 | Building_Update | 2 |
| 0042e980 | Building_InitFromType | 17 |
| 0042ebd0 | Building_Destroy | 5 |
| 0042ed70 | Building_MarkFootprintCells | 6 |
| 0042ef80 | Building_MarkFootprintBuildingFlags | 3 |
| 0042f0c0 | Building_UpdateFootprint | 2 |
| 0042f2a0 | Building_FlattenTerrain | 2 |
| 0042f7c0 | Building_GetAnchorPosition | 34 |
| 0042f850 | Building_ComputeShapeOffset | 99 |
| 0042f8e0 | Building_GetAdjacentPosition | 7 |
| 0042fd70 | Building_OnConstructionComplete | 1 |
| 00430020 | Building_UpdatePopGrowth | 1 |
| 004303f0 | Map_GetCellPtr | 17 |
| 00430430 | Building_UpdateWoodConsumption | 1 |
| 00430960 | Building_UpdateActive_TrainOrSpawn | 1 |
| 00430ef0 | Building_UpdateActive_Convert | 1 |
| 00431970 | Building_UpdateActive_Vehicle | 1 |
| 00432200 | Effect_SetObjectShape | 1 |
| 004322b0 | Building_UpdateConstructing | 1 |
| 004323d0 | Building_UpdateSinking | 1 |
| 004324c0 | Building_AcceptOccupant | 2 |
| 00432800 | Building_EjectPerson | 32 |
| 00432bd0 | Building_CreateFireEffects | 9 |
| 004333f0 | Building_CheckTerrainStability | 1 |
| 00433bb0 | Building_OnDestroy | 1 |
| 00433e20 | Building_UpdateDestroying | 1 |
| 00434090 | Building_HasRoomForOccupant | 2 |
| 00434240 | Building_UpdateSmoke | 1 |
| 00434570 | Building_ApplyDamage | 4 |
| 00434610 | Building_CheckFireDamage | 1 |
| 004348f0 | Building_CheckOccupantStatus | 4 |
| 00434f20 | Building_CheckOccupants | 1 |
| 00435240 | Building_FindAndEjectPerson | 2 |
| 00435430 | Building_Destroy | 2 |
| 00436340 | Building_ResetFireEffects | 3 |
| 004364e0 | Building_InitModelSelector | 1 |
| 00436690 | Building_SpawnDebris | 3 |
| 00437860 | Building_TriggerReconversion | 3 |
| 00438610 | Building_ProcessFightingPersons | 1 |
| 004390d0 | Building_EvaluateFighters | 1 |
| 00439370 | Building_PositionEjectedFighter | 2 |
| 0043da80 | Building_CalcFightPosition | 3 |
| 00490a10 | Building_PlaceOnTerrain | 12 |
| 00491770 | Building_GetFootprintCells | 13 |
| 00491840 | Building_FindPlacementCell | 35 |
| 00491b40 | Building_UpdateDamageLevel | 7 |
| 0047c230 | Building_AssignObjectToSlot | 41 |
| 0047dc60 | Building_UpdateSlotPosition | 38 |
| 004a4fe0 | Building_GatherWoodFromScenery | 2 |
| 0049a030 | Building_BoardPersonOnVehicle | 5 |
| 0049a700 | Building_UpdateVehiclePassengers | 5 |
| 0049b790 | Building_CheckExitAvailable | 5 |
| 004b5990 | Building_ValidatePlacement | 7 |
| 004bdd10 | Building_DestroyOrRubble | 7 |
| 004c3890 | Building_CreateSmokeEffect | 8 |
| 004eeee0 | Building_UpdateConstructionState | 2 |
| 00495c90 | Building_ProcessSpawnState | 21 |
| 00496360 | Building_ProcessProduction | 10 |
| 00496920 | Building_CheckAvailability | 21 |
| 00496ac0 | Building_GetCount | 14 |

## Camera (22 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00421c70 | Camera_SetViewportOffsets | 3 |
| 00422130 | Camera_Initialize | 29 |
| 00422330 | Camera_UpdateFrontendTransition | 2 |
| 004227a0 | Camera_UpdateZoom | 5 |
| 004230a0 | Camera_ProcessFrontendAutoScroll | 2 |
| 00423780 | Camera_ProcessViewModeTransition | 2 |
| 00435e90 | Camera_SetViewMode_A | 1 |
| 0045ae50 | Camera_GetYawRotation | 8 |
| 00469120 | Camera_InitViewTransition | 7 |
| 0046ea30 | Camera_ProjectVertexWithClip | 20 |
| 0046edb0 | Terrain_InitRenderState | 1 |
| 0046f1e0 | Camera_GenerateProjectionLUT | 7 |
| 0046f2a0 | Camera_ApplyRotation | 4 |
| 004909d0 | Camera_ApplyMode | 14 |
| 00497060 | Camera_SetViewMode_B | 1 |
| 004bbd30 | Camera_UpdateOrientation | 6 |
| 004c3cf0 | Camera_GetViewportCoords | 8 |
| 004d3900 | Camera_TransformPoint | 20 |
| 0048b860 | Camera_SetViewMode | 53 |
| 0048bc40 | Camera_SetFrontendMode | 10 |

## Campaign (5 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0041a7c0 | Campaign_LoadLevel | 14 |
| 004214c0 | Campaign_SetCurrentLevelIndex | 3 |
| 00421500 | Campaign_AdvanceToNextLevel | 2 |
| 004219f0 | Campaign_SelectFirstAvailableLevel | 1 |
| 00421aa0 | Campaign_SelectLevelByID | 1 |

## Combat (5 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00437c90 | Combat_ProcessFight | 2 |
| 004b13a0 | Combat_ResetFlags | 2 |
| 004c5d20 | Combat_ProcessMeleeDamage | 1 |
| 004d7490 | Combat_ApplyKnockback | 11 |
| 004d7be0 | Combat_ClearFlags | 39 |

## Config (8 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0041d0f0 | Config_WriteDataWithVersion | 3 |
| 0041d190 | Config_ReadViewportFile | 1 |
| 0041d370 | Config_LoadFromFile | 2 |
| 0041d6c0 | Config_SaveToFile | 1 |
| 00421e20 | Config_LoadViewportSettings | 1 |
| 00421f60 | Config_SaveViewportSettings | 1 |
| 004ba50c | Config_CloseRegistryKey | 2 |
| 004c3e2f | Config_CloseKey | 1 |

## Creature (8 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00483270 | Creature_Init | 1 |
| 00483580 | Creature_SetState | 2 |
| 00484490 | Creature_OrchestrateGroupCombat | 1 |
| 00484770 | Creature_ValidateTarget | 1 |
| 00485900 | Creature_SetupObject | 1 |
| 00485b30 | Creature_ResolveShapeVariant | 7 |
| 004865d0 | Object_DestroyWrapper | 1 |

## DDraw / Display (17 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0041f500 | DDraw_EnumerateDevices | 1 |
| 0050edc0 | DirectDrawCreate | 2 |
| 00510110 | ddraw_cleanup | 11 |
| 00510210 | ddraw_get_display_width | 22 |
| 00510220 | DDraw_LogTodo | 1 |
| 005102e0 | ddraw_init_display | 4 |
| 00510940 | DDraw_Flip | 7 |
| 00510a90 | DDraw_BlitRect | 2 |
| 00510b70 | DDraw_FlipAndClear | 2 |
| 00510c70 | DDraw_Create | 2 |
| 00510ca0 | DDraw_Initialize | 1 |
| 00510e10 | DDraw_RegisterWindowClass | 2 |
| 00511e50 | DDraw_RestoreSurface | 7 |
| 00511e80 | DDraw_ClearSurface | 7 |
| 00512310 | ddraw_lock_surface | 15 |
| 005123a0 | Display_ResetSurface | 15 |
| 0052bb60 | DDraw_DispatchEventLocked | 12 |

## Debug (13 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00427350 | Debug_ResetLevel | 2 |
| 0046a640 | Debug_FindNearestObjectByType | 2 |
| 0046a870 | Debug_FindNearestObjectAllTribes | 2 |
| 00492020 | Debug_DestroyObjectsAtBuildingSite | 2 |
| 004921c0 | Debug_CheatKillObjectList | 1 |
| 004a7b10 | Debug_ProcessCheatCommand | 2 |
| 004ac160 | Debug_WriteSyncChecksum | 2 |
| 004ac480 | Debug_LoadGameSave | 1 |
| 004ac740 | Debug_ProcessMenuItem | 1 |
| 004adb50 | Debug_SelectObjectAtPosition | 1 |
| 004ae0e0 | Debug_ProcessObjectCommand | 1 |
| 004de9d0 | Debug_ToggleOverlay | 5 |
| 004e61c0 | Debug_SetFogOverride | 4 |
| 0050e0a0 | Debug_SelectLinkedObjects | 4 |
| 00496080 | Debug_SpawnObjects | 1 |

## Discovery (2 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 004bec80 | Discovery_Init | 1 |
| 004bedb0 | Discovery_UpdateEffect | 2 |

## Effect (47 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00417ac0 | Spell_AllocateEffect | 11 |
| 00453780 | Effect_QueueVisual | 15 |
| 00453a10 | Effect_SortQueue | 2 |
| 00453cb0 | Effect_CalcLightIntensity | 1 |
| 00453e50 | Effect_ComputeFaceColorTable | 5 |
| 0045b930 | Effect_TriggerCinematic | 7 |
| 0047c150 | Effect_AllocateSlot | 24 |
| 004a6f50 | Effect_Init | 1 |
| 0049c290 | Effect_InitArmageddon | 1 |
| 0049de30 | Effect_InitStateLightning | 1 |
| 0049de60 | Effect_InitStateTornado | 1 |
| 0049e110 | Effect_Update | 1 |
| 0049ea70 | Effect_InitStateEarthquake | 1 |
| 004f0e20 | Effect_Init | 1 |
| 004f1950 | Effect_SetState | 2 |
| 004f2840 | Effect_InitBurn | 3 |
| 004f2ee0 | Effect_InitGroundedObject | 1 |
| 004f3170 | Effect_InitBlast | 2 |
| 004f3260 | Effect_InitTerrainVisual | 1 |
| 004f3360 | Effect_InitGroundScorch | 2 |
| 004f3590 | Effect_InitConversion | 1 |
| 004f3620 | Effect_InitTerrainEffect | 1 |
| 004f36a0 | Effect_InitTerrainParticle | 1 |
| 004f38a0 | Effect_SpawnSubObject | 1 |
| 004f3990 | Effect_InitWithSound | 1 |
| 004f40b0 | Effect_SnapToTerrain | 1 |
| 004f4b20 | Effect_InitRandomState | 1 |
| 004f5bf0 | Effect_InitSoundAtTerrain | 1 |
| 004f7070 | Effect_DestroyWithSound | 1 |
| 004f7230 | Effect_InitRandomTerrain | 1 |
| 004f8550 | Effect_InitRandomized | 1 |
| 004f8a10 | Effect_InitFirestormEffect | 1 |
| 004f8be0 | Effect_InitShapedEffect | 1 |
| 004f8c70 | Effect_InitAtTerrainHeight | 1 |
| 004f9100 | Effect_InitTerrainSnap | 1 |
| 004f93d0 | Effect_InitWithSoundEffect | 1 |
| 004f94f0 | Effect_InitSoundState | 1 |
| 004f98b0 | Effect_InitSoundTerrain | 1 |
| 004f9c10 | Effect_InitTerrainAligned | 1 |
| 004fa6d0 | Effect_InitFlattenTerrain | 1 |
| 004fa830 | Effect_InitShapedTerrain | 1 |
| 004fa8e0 | Effect_InitSoundShape | 1 |
| 004fa9d0 | Effect_InitBasicTerrain | 1 |
| 004faa40 | Effect_InitSoundBasic | 1 |
| 004fac00 | Effect_InitStateA | 1 |
| 004fac70 | Effect_InitStateB | 1 |
| 004face0 | Effect_InitStateC | 1 |
| 004fad50 | Effect_InitStateD | 1 |
| 004faf30 | Effect_ApplyTerrainCrater | 2 |
| 004fb120 | Effect_SpawnRandomObjects | 1 |
| 004fb320 | Effect_DestroyByType | 1 |
| 004fb450 | Effect_SpawnTerrainObject | 1 |
| 004fb540 | Effect_SpawnMovingObject | 1 |
| 004fbb80 | Effect_InitSoundStateAlt | 1 |
| 004fbea0 | Effect_InitShaped | 1 |
| 00500750 | Effect_SpawnExplosions | 8 |

## Entity (3 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00401090 | Entity_ReturnToPool | 14 |
| 00483550 | Entity_UpdateHeight | 12 |
| 00497fb0 | Entity_ClampToTerrain | 5 |

## File I/O (22 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0041d230 | File_ValidateHeader | 4 |
| 0041e850 | File_LoadDataEntry | 4 |
| 004c41a0 | CRT_Strcpy | 17 |
| 004c4310 | BuildFilePath | 107 |
| 004c4140 | BuildBasePath | 53 |
| 005113a0 | File_Open | 5 |
| 00511410 | File_Close | 76 |
| 00511430 | File_GetPosition | 1 |
| 00511450 | File_Seek | 4 |
| 005114b0 | File_GetSize | 3 |
| 005114c0 | File_GetSize | 3 |
| 00511520 | File_Exists | 32 |
| 005115a0 | File_DeleteResolved | 16 |
| 005115e0 | File_Move | 3 |
| 00511600 | File_Copy | 4 |
| 00511620 | File_Read | 33 |
| 00511680 | File_Write | 15 |
| 005116d0 | File_SetWorkingDir | 3 |
| 00511730 | File_GetWorkingDir | 3 |
| 00511830 | File_ResolvePath | 15 |
| 005119b0 | File_ReadEntire | 64 |
| 00511a80 | File_LoadIntoBuffer | 9 |
| 00511be0 | File_ParseWorkingDirFromCmdLine | 1 |
| 00511cb0 | File_OpenWrapper | 29 |
| 0052d830 | File_CloseHandle | 11 |

## Font / Text (16 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0040a750 | Font_GetMetadata | 1 |
| 0041cfd0 | Font_GetStringWidth | 49 |
| 004a02b0 | Font_SetCurrentSize | 62 |
| 004a0310 | Font_RenderString | 68 |
| 004a0420 | Font_RenderSmallChar | 2 |
| 004a0570 | Render_DrawCharacter | 2 |
| 004a07c0 | Font_RenderLargeChar | 2 |
| 004a0d60 | Font_GetWidth16bit | 82 |
| 004a1d00 | Font_ConvertEncoding | 31 |
| 004a1f20 | Language_FormatString | 21 |
| 004a20b0 | Font_LoadFiles | 1 |
| 004a2230 | Font_UnloadAll | 2 |
| 0050fae0 | Font_DrawAtPosition8bit | 67 |
| 0050fc20 | Font_Render8bit | 59 |
| 0050fcc0 | Font_GetWidth8bit | 84 |
| 004d2c60 | Text_TruncateToFit | 10 |

## Formation (1 function)

| Address | Name | XRefs |
|---------|------|-------|
| 004ee500 | Formation_ReorderUnits | 1 |

## Frontend (2 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00411900 | Frontend_UpdateCutsceneSequences | 2 |
| 004dea50 | Frontend_Init | 2 |

## Game / GameLoop / GameState (48 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0040e160 | GameState_CheckLevelFileExists | 2 |
| 0041fab0 | GameState_Loading | 1 |
| 0041fc50 | GameState_InitLoading | 2 |
| 0041fd60 | GameState_Loading_MainTick | 1 |
| 00419040 | GameLoop_Shutdown | 1 |
| 004170c0 | GameLoop_CheckDisplayModeChange | 1 |
| 0041f3c0 | GameLoop_UpdateTiming | 1 |
| 00422a20 | Game_SetAlignmentMode | 21 |
| 00423c60 | Game_CheckVictoryConditions | 1 |
| 00426440 | Game_TriggerVictory | 3 |
| 00426500 | GameState_ProcessMenuTransition | 1 |
| 004266e0 | Game_TriggerVictoryForTribe | 3 |
| 00426e80 | GameState_PlayRandomAmbientSound | 1 |
| 00427c60 | Render_FinalDisplay | 1 |
| 0043f900 | GameState_ProcessSoundQueue | 1 |
| 0044fed0 | GameState_LoadLevelData | 3 |
| 0044ff20 | GameState_LoadFrontendAssets | 3 |
| 004517a0 | GameState_ResetLevel | 7 |
| 00457340 | GameState_RequestTransition | 3 |
| 00486f70 | GameState_ResetTopmap | 1 |
| 004907d0 | GameState_UpdateBuildingPlacement | 1 |
| 004abc80 | GameState_InitiateTransition | 27 |
| 004abcd0 | GameState_CompleteTransition | 5 |
| 004abd00 | GameState_SetState | 10 |
| 004ba520 | GameLoop | 1 |
| 004baa40 | GameState_Frontend | 1 |
| 004bae70 | GameState_Outro | 1 |
| 004bafe0 | GameLoop_LoadResources | 1 |
| 004bb380 | GameLoop_ParseCommandLine | 2 |
| 004bb5a0 | Game_SimulationTick | 1 |
| 004bbdc0 | GameState_AnimateFrontendCamera | 1 |
| 004bc730 | GameMain | 1 |
| 004b9fd0 | GameLoop_CheckRunningInstance | 1 |
| 004c03d0 | GameState_Multiplayer | 1 |
| 004c0720 | GameState_RenderMultiplayerText | 1 |
| 004c0a00 | GameState_LoadCreditsFile | 1 |
| 004c4af0 | Game_FatalExit | 9 |
| 004c4c20 | Game_ProcessInput | 1 |
| 004c59d0 | GameLoop_ValidateCDCheck | 4 |
| 004c5b30 | GameLoop_MeasureFrameTime | 1 |
| 004d15a0 | GameState_ExitInGame | 4 |
| 004ddd20 | GameState_InGame | 1 |
| 004de3f0 | GameState_UpdateFrontendTimer | 1 |
| 004de470 | GameState_UpdateRenderFrame | 1 |
| 004dee90 | GameState_UpdateRenderFrame | 4 |
| 004ea0e0 | Game_UpdateUI | 1 |
| 004e7020 | Game_SendVictoryMessage | 1 |
| 004a6be0 | Game_RenderEffects_Stub | 1 |
| 0048c070 | Game_RenderWorld | 1 |
| 0048b210 | Game_ProcessCommand | 106 |
| 004c3e40 | GameLoop_SetupDirectoryPaths | 1 |
| 00513040 | GameLoop_SetDeveloperString | 1 |
| 00419d10 | Render_ResetScreen | 1 |
| 00419d30 | GameState_RenderLoadingScreen | 1 |

## General (7 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0045fe00 | General_Init | 1 |
| 004600c0 | General_SetState | 2 |
| 004601c0 | Object_DispatchStateUpdate | 1 |
| 004602b0 | General_MoveRelativeToBuilding | 2 |
| 004603c0 | General_CreateObject | 1 |
| 00460aa0 | General_CreateSinkingObject | 1 |
| 00461ae0 | General_CalcHeightOnBuilding | 1 |
| 00461c10 | General_CreateScenery | 1 |

## GUI (18 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0040aaa0 | GUI_GetBufferForLayer | 7 |
| 0040b150 | GUI_DrawStringWithShadow | 1 |
| 0040b2b0 | GUI_DrawString | 16 |
| 0040b5f0 | GUI_DrawStringAligned | 6 |
| 0040b930 | GUI_MeasureString | 11 |
| 0040ba20 | GUI_TestElementVisibility | 5 |
| 0040ba80 | GUI_ClipStringToWidth | 2 |
| 0040bb90 | GUI_TruncateStringFromEnd | 3 |
| 0040bc60 | GUI_TruncateStringFromStart | 3 |
| 0041a370 | GUI_BlitTiledBorder | 1 |
| 004936b0 | GUI_RenderTiledPanel | 11 |
| 004df6c0 | GUI_LayoutGrid | 3 |
| 004df820 | GUI_LayoutGridWithRender | 1 |
| 004df9c0 | GUI_RenderTiledElement | 3 |
| 004dfab0 | GUI_RenderScaledElement | 1 |
| 004e0000 | draw_sprite_rle | 3 |
| 004e0110 | GUI_RenderRLESprite8bpp | 2 |
| 004e1960 | GUI_RenderSceneForTurn | 1 |
| 004e1980 | GUI_RenderSceneElement | 3 |

## Input (16 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 004163f0 | Input_SelectAtCursor | 1 |
| 004236c0 | Input_HandleGameAction | 14 |
| 00464830 | Input_ProcessKeyCommand | 11 |
| 00469ce0 | Input_PlaySelectSound | 9 |
| 0048b000 | Input_ProcessKeyEvent | 73 |
| 0048c0e0 | Input_SelectObjectAtCursor | 3 |
| 0048c4b0 | Input_ProcessObjectSelection | 2 |
| 0049ef20 | GameState_RenderKeyBindingScreen | 1 |
| 0049f7e0 | Input_SetKeyBinding | 2 |
| 0049fcc0 | Input_ParseKeyDefFile | 2 |
| 004a0010 | Input_BuildKeyBindingTable | 1 |
| 004a70c0 | Input_ProcessCameraSpellCommand | 2 |
| 004ab6e0 | Input_ApplyCameraMovement | 1 |
| 004acea0 | Input_UpdateCameraSpellView | 4 |
| 004dbd20 | Input_LoadKeyDefinitions | 1 |

## Internal (8 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00437c30 | Internal_PlaceObjectInWorld | 1 |
| 004ecf50 | Internal_Init | 1 |
| 004ed340 | Internal_SetState | 2 |
| 004ed3e0 | Object_UpdateHeightByState | 1 |
| 004ed510 | Object_UpdateMovement | 1 |
| 004ee970 | Internal_CreateRandomObject | 1 |
| 004eecf0 | Internal_CreateTerrainObject | 1 |
| 004eedc0 | Internal_Update | 1 |
| 004eee20 | Internal_CreateOrDestroyObject | 1 |

## Interpolation (2 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0041cc90 | Interpolation_StartChannel | 34 |
| 0041cd90 | Interpolation_StepChannel | 46 |

## Language (7 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0040b8c0 | Language_Cleanup | 19 |
| 00450e70 | Language_LoadFontSprites | 2 |
| 00453030 | Language_LoadStrings | 1 |
| 004531c0 | Language_SetCurrent | 1 |
| 00453380 | Language_CloseRegistryKey | 1 |
| 004dfe10 | Language_LoadFrontendSprites | 2 |

## Level (10 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0040c330 | Level_LoadAndCreateObjects | 1 |
| 0040cc10 | LoadLevelHeader | 2 |
| 0040cde0 | LoadLevelData | 1 |
| 0040cf80 | LoadLevelData | 7 |
| 0040d210 | Level_StartLevel | 2 |
| 0040d420 | Level_PostCreateUnit | 1 |
| 0040dc70 | LoadLevelSpecialData | 1 |
| 0040dd70 | LoadObjectivesData | 1 |
| 0040de70 | LoadAIScripts | 1 |
| 0040dfc0 | Level_SpawnBuildingsFromGenerals | 1 |
| 0040e230 | Level_LoadFileByNumber | 4 |
| 0041d290 | LoadLevelObjectCount | 2 |
| 00421320 | LoadLevelTextures | 2 |
| 0041eb50 | LoadConstantsDat | 1 |
| 00420d90 | LevelSelect_UpdateCameraAndNodes | 1 |
| 004528e0 | Level_InitializeSession | 4 |
| 0041f4a0 | Level_ClearScreen | 1 |
| 0049de90 | Level_PrepareBuildingSite | 2 |
| 00462680 | Level_WriteSaveGame | 1 |

## Math (10 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0041f0c0 | Math_CalculateWrappedDistanceSquared | 13 |
| 0041f230 | Math_SpiralPosition | 20 |
| 00439900 | Math_IsWithinRange | 16 |
| 00473a50 | Buffer_ClearRegion | 2 |
| 004d4b20 | Math_MovePointByAngle | 119 |
| 004d7c10 | Math_AngleDifference | 23 |
| 004d7c40 | Math_GetRotationDirection | 16 |
| 004ea8f0 | Math_DistanceWrapped | 45 |
| 004ea950 | Math_DistanceSquared | 41 |
| 00564000 | Math_IntegerSqrt | 20 |
| 00564074 | Math_Atan2 | 141 |

## Matrix / Vector (6 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00450320 | Matrix3x3_Identity | 15 |
| 004bc000 | Vector3_TransformByRow | 2 |
| 004bc060 | Matrix3x3_Multiply | 3 |
| 004bc1e0 | Matrix3x3_RotateX | 11 |
| 004bc260 | Matrix3x3_RotateY | 3 |
| 004bc2e0 | Matrix3x3_RotateZ | 8 |
| 004bc360 | Matrix3x3_RotateArbitrary | 13 |

## Minimap (5 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0042b900 | Sprite_FreeBank | 4 |
| 0042b950 | Minimap_Update | 1 |
| 0042ba10 | Minimap_RenderTerrain | 1 |
| 0042bbe0 | Minimap_RenderObjects | 1 |
| 0042bff0 | Minimap_UpdateDirtyRegion | 1 |
| 0045aa50 | Minimap_GetBounds | 4 |
| 00494cf0 | Minimap_DrawSprite | 12 |

## Model3D (9 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00471730 | Model3D_RenderObject | 1 |
| 00472720 | Model3D_SubmitTriFace | 7 |
| 004728d0 | Model3D_SubmitTriFaceTinted | 2 |
| 00476330 | Model3D_SubmitShadow | 2 |
| 00476430 | Model3D_SetupSelectionBuffer | 2 |
| 004765a0 | Model3D_DrawSelectionEdge | 5 |
| 00476690 | Model3D_SubmitSelectionHighlight | 2 |
| 00477640 | Model3D_ApplyVertexWind | 2 |
| 0049c020 | Model3D_ComputeFaceNormal | 6 |

## Network (22 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 004ac030 | Network_RotateSyncLogFiles | 2 |
| 004ae450 | Network_LoadConfig | 2 |
| 004cd590 | Network_EnableNewPlayers | 1 |
| 004e3ae0 | Network_SendTrackedPacket | 2 |
| 004e3bc0 | Network_CopyTimedData | 1 |
| 004e4b40 | Network_Initialize | 9 |
| 004e4ce0 | Network_Shutdown | 10 |
| 004e5040 | Network_QueueReceivedMessage | 2 |
| 004e5450 | Network_QueueSyncMessage | 2 |
| 004e57a0 | Network_OpenSyncLog | 3 |
| 004e5ad0 | Network_WriteSyncLog | 2 |
| 004e6050 | Network_ClearSyncLog | 6 |
| 004e6130 | Network_ResetMessageQueues | 2 |
| 004e6300 | Network_CompleteResync | 1 |
| 004e6550 | Network_AllocateBuffer | 1 |
| 004e6b20 | Network_RefreshPlayerInfo | 7 |
| 004e6c40 | Network_SendPacket | 23 |
| 004e6d00 | Network_UpdatePlayerStatus | 4 |
| 004e6f50 | Network_SendPacket_A | 1 |
| 004e6f80 | Network_SendPacket_B | 1 |
| 004e6fb0 | Network_SendStateUpdate | 14 |
| 004e70c0 | Network_ConvertPlayerStrings | 2 |

## Object (32 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00401000 | Object_ChangeState | 41 |
| 00401200 | Object_SnapToGrid | 16 |
| 00411040 | Object_SelectForRendering | 16 |
| 0046ac10 | Object_IsWithinRange | 13 |
| 00470030 | Object_SubmitToDepthBucket | 2 |
| 00476770 | Object_RenderHighlight | 1 |
| 00483a50 | Object_TickStateMachine | 1 |
| 004991170 | Object_TestSelectable | 6 |
| 00499eb0 | Object_IsValidPosition | 19 |
| 004ae8d0 | Person_SetSelectionState | 13 |
| 004aea50 | Object_SetSelected | 36 |
| 004aee90 | Tribe_UpdateObjectStats | 12 |
| 004af950 | Object_InitByType | 3 |
| 004afa10 | Object_SetStateByType | 271 |
| 004afac0 | Object_ClearStateByType_Stub | 277 |
| 004afad0 | Object_UpdateState | 3 |
| 004afbf0 | InitObjectPointerArray | 1 |
| 004afc70 | Object_Create | 188 |
| 004affa0 | Object_Allocate | 18 |
| 004b00c0 | Object_Destroy | 101 |
| 004b01e0 | Object_CopyData | 3 |
| 004b0320 | Object_ProcessTransports | 1 |
| 004b0560 | Object_RemoveFromTracker | 16 |
| 004b0840 | Object_LinkToCell | 41 |
| 004b08c0 | Object_UnlinkFromTerrainCell | 19 |
| 004b0950 | Object_MoveToPosition | 79 |
| 004b1550 | Object_DestroyByType | 121 |
| 004b42a0 | Object_SetType | 29 |
| 004b42c0 | Object_IsType | 30 |
| 004b4320 | Object_SetSubState | 14 |
| 004b4eb0 | Object_IsWithinRadius | 15 |
| 004d4db0 | Object_UpdateFlyingPhysics | 4 |
| 004d7200 | Object_HandleLanding | 2 |
| 004d77c0 | Object_HandleFlyingCollision | 1 |
| 004d79f0 | Object_CalculateFlyingAngle | 1 |
| 004d8e60 | Object_UpdateRouteMovement | 3 |
| 004edc10 | Object_CalculateMovementTrail | 3 |
| 004ee590 | Object_DestroySimple | 1 |
| 004fe140 | Object_ProcessPersonState | 1 |
| 00454050 | GetObjectTypeName | 1 |
| 004bd370 | Object_ProcessStateTransition | 1 |
| 004bd5b0 | Object_InitShapeData | 16 |
| 00504f20 | Object_ApplyDamage | 12 |
| 0050e2e0 | Object_IsVisible | 18 |
| 00509650 | Object_MatchesSubType | 37 |
| 004d7e20 | Person_UpdateTargetPosition | 65 |
| 004d7e70 | Person_SetDestination | 42 |
| 004d8500 | Path_CleanupResources | 79 |
| 00496d00 | Object_CanBuild | 16 |

## Palette (7 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 004503f0 | Palette_InitializePaths | 4 |
| 00450730 | SaveGame_RestorePalette | 3 |
| 00450790 | Palette_LoadForLevel | 1 |
| 00450fc0 | Palette_BuildColorIndexTable | 5 |
| 004c5810 | Palette_ApplyBrightness | 9 |
| 00510ba0 | Palette_SetEntries | 23 |
| 0050f7f0 | Palette_FindClosestColor | 29 |

## Path (5 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0041f140 | Path_GatherCellsAlongRoute | 3 |
| 00422ba0 | Path_ComputeSteeringForces | 7 |
| 004248c0 | Path_UpdateDirection | 1 |
| 00424ed0 | Path_FindBestDirection | 1 |
| 004fec90 | Path_Cleanup | 1 |

## Person (32 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00437ae0 | Person_InitGroundPosition | 1 |
| 00437b40 | Person_EnterFightingState | 1 |
| 004350b0 | Person_CheckBuildingExit | 1 |
| 0043f2a0 | Person_CreateCelebrationEffects | 3 |
| 00477810 | Person_HandleShieldExit | 1 |
| 0047ba00 | Person_RemoveTimedEffect | 33 |
| 0047c1d0 | Person_ClearAllEffects | 33 |
| 004807e0 | Person_ApplyTimedEffect | 26 |
| 004b1050 | Person_SetFacingToBuilding | 5 |
| 004b1220 | Person_SetHeadingToHome | 1 |
| 004b14d0 | Person_SetIdleState | 3 |
| 004be8f0 | Person_AdjustHealth | 10 |
| 0049b4e0 | Person_CheckShieldOnDeath | 14 |
| 0049d5d0 | Person_CalcVehicleSeatPosition | 4 |
| 004fd260 | Person_Init | 1 |
| 004fd5d0 | Person_SetState | 2 |
| 004fe080 | Person_SetStateAnimation | 2 |
| 004fe0e0 | Person_AnimateState | 1 |
| 004fed30 | Person_SelectAnimation | 16 |
| 004fee80 | Person_SetAnimationByState | 41 |
| 004feed0 | Person_SetAnimation | 79 |
| 004ff9e0 | Person_DestroyAndCleanup | 7 |
| 004ffd70 | Person_SetIdleAnimation | 30 |
| 004ffdd0 | Person_SetRandomAnimation | 55 |
| 005007b0 | Person_InitCommon | 8 |
| 00500b00 | Person_EnterMovingState | 1 |
| 00501750 | Person_EnterBuildingState | 3 |
| 00501c00 | Person_EnterTrainingState | 1 |
| 00501e20 | Person_EnterHousingState | 1 |
| 00502160 | Person_SnapToTerrainHeight | 1 |
| 005021c0 | Person_EnterGatheringState | 1 |
| 00502f70 | Person_StartWoodGathering | 3 |
| 00503190 | Person_EnterDrowningState | 1 |
| 00503e50 | Person_EnterPreachingState | 1 |
| 00504410 | Person_EnterBeingConvertedState | 1 |
| 00505010 | Person_PlayFightSound | 5 |
| 0050a960 | Person_EnterVehicleState | 1 |
| 0050b480 | Person_ExitVehicleState | 1 |
| 0050b990 | Person_EnterCelebrationState | 2 |
| 0050d4c0 | Person_SetCelebrationAnim | 2 |
| 0050d620 | Person_EnterTeleportState | 1 |

## Preacher / Wild (2 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00509e90 | Preacher_StartConverting | 1 |
| 00502e60 | Wild_ConvertToBrave | 2 |

## Projection (2 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0046ed30 | Projection_InitializeDefaults | 1 |
| 0046f490 | Projection_SetFromParams | 3 |

## Registry (5 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00513660 | Registry_OpenBullfrogKey | 5 |
| 00513790 | Registry_CloseKey | 5 |
| 005137a0 | Registry_CloseKey | 9 |
| 005137e0 | Registry_OpenKey | 1 |
| 00513900 | Registry_ReadValue | 5 |
| 004ba35b | Registry_CloseKey_Thunk | 1 |
| 004ba370 | InitConfigFromRegistry | 1 |

## Render (68 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0040a560 | Render_DrawRotatedQuad | 1 |
| 0040ab40 | Render_DrawTextOverlays | 3 |
| 0040aea0 | Render_DrawTextOverlays_Prev | 1 |
| 0040b060 | Render_SaveTextCmdBufferPos | 23 |
| 0040b070 | Render_DrawCenteredText | 3 |
| 0041c2a0 | Render_CheckFlag | 39 |
| 0041f9b0 | RenderFlags_SetDirtyBit | 10 |
| 0041fa30 | Render_SetScreenOrigin | 10 |
| 004533e0 | Render_Is16bppMode | 325 |
| 004502a0 | Render_InitGlobals | 1 |
| 00464190 | Render_SetupDisplay | 1 |
| 00467680 | Render_SetupTerrainEffects | 1 |
| 00467890 | Render_PostProcessEffects | 1 |
| 00468290 | Render_ProcessChatInput | 3 |
| 00468c30 | SaveGame_SyncRenderState | 2 |
| 00468d80 | Render_ResetBuffers | 19 |
| 00468ea0 | Render_ProcessFrame | 38 |
| 00429a50 | Render_SetupRasterizerCallback | 1 |
| 00427d10 | UI_RenderViewport | 1 |
| 00463900 | Render_ProcessInputAndDraw | 6 |
| 0046af00 | Render_ProcessDepthBuckets_Main | 1 |
| 0046d9a0 | Render_ProcessDepthBuckets_3DModels | 2 |
| 00470210 | Render_SubmitGrassPatches | 1 |
| 004707c0 | Render_SubmitHealthBar | 5 |
| 00475f50 | Render_DrawGroundCircle | 1 |
| 004760d0 | Render_DrawGroundCircleAnimated | 2 |
| 00476890 | Render_DrawShadowBlob | 1 |
| 00476c40 | Render_DrawShadowProjection | 1 |
| 00476e40 | Render_ProcessSelectionHighlights | 1 |
| 004771a0 | Render_ProcessUnitMarkers | 1 |
| 00477420 | Render_CalculateDistanceScale | 22 |
| 00487e30 | Render_Process3DModels | 1 |
| 0048b0e0 | Render_SetPostProcessEffect | 27 |
| 004a0230 | Render_DrawLayer | 1 |
| 004a0d40 | Render_GetColorMode | 43 |
| 004a6bf0 | DrawFrameRate | 4 |
| 004a6d30 | Render_HandleScreenshot | 3 |
| 004abab0 | Render_ApplyScreenShakeX | 2 |
| 004abb50 | Render_ApplyScreenShakeY | 2 |
| 004ad210 | Terrain_CalculateRotatedQuad | 2 |
| 004ad3e0 | Terrain_CalculateBoundingRect | 2 |
| 004c46d0 | Render_InitDisplayMode | 11 |
| 004c55d0 | Render_UpdateFrameCounter | 108 |
| 004dc3c0 | Render_ResetState | 1 |
| 004de480 | UI_RenderHUDOverlay | 1 |
| 004dedc0 | Render_SubmitSimpleCmd | 1 |
| 004df3c0 | render_frame | 1 |
| 004e7660 | Render_DrawSpriteBucket | 2 |
| 004e7b90 | Render_DrawSpritesInBucket | 3 |
| 004e8210 | Render_SaveTextBuffer | 3 |
| 0047c540 | Render_SelectObjectLayer | 3 |
| 0047cc80 | Render_BuildLayerOrder | 1 |
| 0047d620 | Render_DetermineObjectSelectable | 1 |
| 004c3b40 | Render_SetupClipRect | 8 |
| 004c3bb0 | Render_SetClipRect | 20 |
| 00494b90 | Render_DrawSpriteBox | 1 |
| 0050f110 | Render_BlitRect | 21 |
| 0050f300 | Render_SetClipRectAndTarget | 6 |
| 0050f390 | Render_SetupViewportClipping | 4 |
| 0050f510 | Render_SetViewportClip | 1 |
| 0050f520 | Render_SetBitDepthVtable | 5 |
| 0050f5f0 | Render_SetupColorMasks | 20 |
| 0050fc90 | Render_CallInterfaceMethod | 42 |
| 0052b8c0 | Render_GetClipX | 14 |
| 0052b8d0 | Render_GetClipY | 14 |
| 0052b950 | Render_CalculateBufferOffset | 2 |
| 0052b990 | Render_SetupBitMask | 4 |
| 0052b9e0 | Render_SetupBitMasks | 5 |
| 004d0bd0 | Render_FormatResolutionText | 1 |
| 0046f6e0 | UI_SetClipRect | 1 |
| 0097c000 | Rasterizer_Main | 15 |

## RenderCmd (22 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00512760 | RenderCmd_ReadNext | 9 |
| 00512860 | RenderCmd_GetCount | 2 |
| 00512900 | Render_InitScreen | 3 |
| 00512920 | Render_GetViewport | 3 |
| 00512930 | RenderCmd_SubmitSimple | 3 |
| 005129ca | RenderCmd_SignalSimple | 2 |
| 005129e0 | RenderCmd_SubmitComplex | 23 |
| 00512b14 | RenderCmd_ReleaseSemaphore | 3 |
| 00512b40 | Render_InitScreenBuffer | 24 |
| 00512b50 | RenderCmd_SubmitSprite | 7 |
| 00512bea | RenderCmd_SignalSprite | 2 |
| 005125d0 | Render_ProcessCommandBuffer | 7 |
| 00513000 | RenderCmd_ProcessType2 | 2 |
| 005123c0 | Sprite_BlitWithVtable | 19 |
| 0052c7d0 | RenderCmd_ReadFromBuffer | 1 |
| 0052d380 | RenderCmd_WriteData | 3 |
| 0052d430 | RenderCmd_AllocateBuffer | 2 |
| 0052d4e0 | RenderCmd_DestroyBuffer | 1 |
| 0052d550 | RenderCmd_WriteSpriteData | 2 |
| 0052d580 | RenderCmd_GetViewportBounds | 4 |
| 0052d810 | RenderCmd_CreateSemaphore | 5 |
| 0052d840 | RenderCmd_LockBuffer | 15 |
| 0052d870 | RenderCmd_CheckSpace | 17 |
| 0052d8f0 | RenderCmd_InitBuffer | 1 |
| 0052da70 | RenderCmd_ExecuteComplex | 2 |

## SaveGame (11 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0040d780 | SaveGame_LoadStateFromBuffer | 4 |
| 0040d9e0 | SaveGame_CopyStateToBuffer | 4 |
| 00462130 | SaveGame_Create | 1 |
| 00462340 | SaveGame_SerializeToBackup | 2 |
| 00462430 | SaveGame_DeserializeFromFile | 3 |
| 004627f0 | SaveGame_Save | 3 |
| 00462d00 | SaveGame_Load | 3 |
| 004ac320 | SaveGame_DeleteExistingFiles | 4 |
| 004ae500 | GameState_SaveNetConfig | 3 |
| 00511b60 | SaveGame_WriteBackupFile | 4 |

## Scenery (7 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 004bcde0 | Scenery_Init | 1 |
| 004bd100 | Scenery_SetState | 2 |
| 004bd6c0 | Scenery_PlaceOnTerrain | 3 |
| 004bda10 | Scenery_CreateObject | 1 |
| 004bdbb0 | Scenery_InitializeObject | 1 |
| 004bf760 | Scenery_UpdateHeight | 3 |
| 004bff30 | Scenery_DamageNearbyObjects | 3 |

## Screen (2 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00459570 | Screen_ScaleX | 64 |
| 00459590 | Screen_ScaleY | 63 |

## Shading / IDCT / Shade (10 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00486a20 | Shading_InitializeLookupTables | 0 |
| 00486ca0 | Shading_ComputeLookupTable_A | 4 |
| 00487350 | Shading_AllocateBuffers | 5 |
| 004875a0 | Shading_LoadTerrainTables | 2 |
| 004513b0 | Shading_LoadBackgroundSprite | 2 |
| 0055d316 | Shade_RemapGammaPacked | 16 |
| 0055dde9 | Shade_CombineLightmap32 | 32 |
| 0055e074 | Shade_CombineLightmap16 | 32 |
| 005603c0 | Shade_BlendColor32 | 16 |
| 00560890 | Shade_BlendColor16 | 16 |
| 00560d90 | Shade_LookupTexel8 | 16 |
| 0055d960 | IDCT_DecodeDCTCoefficients | 18 |
| 0056123c | IDCT_ButterflyColumn8 | 104 |
| 0056137b | IDCT_ButterflyRow8 | 96 |

## Shield (2 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0049a230 | Shield_EjectPerson | 17 |
| 0049a9f0 | Shield_FindExitPosition | 16 |

## Shape (8 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00410870 | Shape_ParseSphereFile | 1 |
| 0048f8d0 | Shape_Init | 1 |
| 0048f9b0 | Shape_SetState | 2 |
| 0048fa80 | Shape_PlaceOnTerrain | 1 |
| 0049b990 | Shape_LoadBank | 4 |
| 0049b9b0 | Shape_LoadBankData | 1 |
| 0049bba0 | Shape_LoadDatFile | 1 |
| 0049bc30 | Shape_UnloadDatFile | 1 |
| 0049bc40 | Shape_PatchPointers | 1 |

## Shot (7 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 004573e0 | Shot_Init | 1 |
| 004576f0 | Shot_SetState | 2 |
| 00457c20 | Shot_SpawnProjectile | 1 |
| 004585c0 | Shot_LaunchProjectile | 1 |
| 00458800 | Shot_Update | 1 |
| 00458dd0 | Shot_SpawnProjectile | 2 |
| 0048ebf0 | Shot_CheckShieldBlock | 4 |
| 004eaa20 | Shot_ComputeDistance3D | 5 |
| 004fb620 | Shot_ProcessImpact | 1 |

## Sky (9 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 004dc0e0 | Sky_RenderOrchestrator | 1 |
| 004dc3f0 | Sky_BuildPaletteLUT | 1 |
| 004dc710 | Sky_UpdateRotation | 1 |
| 004dc850 | Sky_ComputeParams | 1 |
| 004dc890 | Sky_SetViewport | 1 |
| 004dc930 | Sky_RasterizeScanline | 1 |
| 004dcc30 | Sky_RenderTiled | 1 |
| 004dd710 | Sky_RenderSimple | 1 |
| 004dd790 | Sky_RenderParallax | 1 |
| 004dd880 | Sky_RenderFlatFill | 1 |

## Sound (17 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00416a20 | Sound_UpdateAmbient | 2 |
| 00416ef0 | Sound_PlayObjectSound | 2 |
| 00417100 | Mana_UpdateCollectionSound | 3 |
| 00417300 | Sound_Play | 211 |
| 00417a20 | Sound_StopByID | 16 |
| 00417bb0 | Sound_UpdateActiveSounds | 3 |
| 00417e60 | Sound_AllocateChannel | 1 |
| 00418000 | Sound_StopChannel | 6 |
| 004183b0 | Sound_Update3DPosition | 2 |
| 00418700 | Sound_UpdateAllChannels | 7 |
| 004187b0 | Sound_LoadDrumTrack | 8 |
| 00418b80 | Sound_FinishPendingPlayback | 7 |
| 00418c00 | Sound_LoadSDT | 1 |
| 00418f40 | Sound_LoadSDTLowQuality | 2 |
| 0048d920 | Sound_AllocateChannel | 10 |
| 004c41d0 | Sound_ResolvePath | 8 |
| 005393a0 | Sound_InitSource | 19 |
| 0053a470 | Sound_CreateSampleByType | 6 |
| 0053a720 | Sound_InitObject | 6 |
| 0053a800 | Sound_InitCDAudio | 1 |
| 0053a9d0 | Sound_SendMCICommand | 1 |
| 0053aa30 | Timer_Kill | 4 |
| 0053bcb0 | Sound_GetDirectSound | 1 |

## Spell (17 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0041c8e0 | Spell_GetIndexFromBit | 1 |
| 00495440 | Spell_Init | 1 |
| 004958b0 | Spell_SetState | 2 |
| 00495b00 | Spell_LaunchAtTarget | 21 |
| 004a5a30 | Spell_FindTargetScenery | 1 |
| 004a5b60 | Spell_CheckTargetValid | 1 |
| 004f1960 | Spell_DispatchEffectUpdate | 2 |
| 004f2550 | Spell_ProcessBurn | 1 |
| 004f2950 | Spell_ProcessShockwave | 1 |
| 004f3a50 | Spell_ProcessBlast | 1 |
| 004f3ee0 | Spell_CreateFirestorm | 2 |
| 004f6480 | Spell_CreateSwarmEffect | 1 |
| 004f7330 | Spell_ProcessLightningSwarm | 1 |
| 004f7f50 | Spell_CreateLightningBolt | 1 |
| 004f8230 | Spell_MoveLightningBolt | 1 |
| 004f8390 | Spell_SteerLightningBolt | 3 |

## Sprite (15 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00402800 | Sprite_SetupScanline8bpp | 234 |
| 00402840 | Sprite_DrawScanline8bpp | 72 |
| 00411b70 | Sprite_RenderWithShadow | 2 |
| 00411c90 | Sprite_RenderObject | 3 |
| 00416110 | Sprite_RenderShadowMask | 1 |
| 0041db20 | Sprite_LoadResources | 5 |
| 0041e790 | Sprite_LoadFromDisk | 6 |
| 0041e7c0 | Sprite_FreeBankData | 11 |
| 00450990 | Sprite_LoadBank | 4 |
| 004507e0 | Sprite_ReloadResources | 1 |
| 00451b50 | Sprite_InitAnimationTables | 5 |
| 00451ff0 | Sprite_SetResolutionParams | 2 |
| 0050edd0 | Sprite_Blit | 92 |
| 0050ef90 | Sprite_SetupScanline16bpp | 56 |
| 0050f050 | Sprite_DrawScanline16bpp | 74 |
| 0050f6e0 | Sprite_BlitScaled | 34 |
| 0050f720 | Sprite_SetupScaledBlit | 1 |
| 005094b0 | Sprite_SetRenderTarget | 7 |
| 005643c0 | Sprite_BlitScaledInternal | 1 |
| 00566438 | Sprite_SetupScaleParams | 1 |
| 0050ef70 | Draw_Pixel | 25 |

## Stats (1 function)

| Address | Name | XRefs |
|---------|------|-------|
| 0041d980 | Stats_AccumulateValue | 44 |

## Script / Event (1 function)

| Address | Name | XRefs |
|---------|------|-------|
| 004e0650 | Script_ExecuteEventList | 28 |

## Terrain (52 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00418570 | Terrain_CalcDistanceToSpecialTile | 2 |
| 0041cf70 | Terrain_ClearCellFlag | 33 |
| 0042c170 | Terrain_ProcessVisibleObjects | 1 |
| 0042d4d0 | Terrain_InterpolateEdgePositions | 4 |
| 0042dc60 | Terrain_SortFaceVertices | 1 |
| 0042dff0 | Terrain_ComputeFaceNormals | 1 |
| 00435760 | Terrain_FindObjectAtPosition | 44 |
| 0045f7f0 | Terrain_FlattenAtPosition | 1 |
| 00459670 | Terrain_SelectLOD | 1 |
| 00406500 | Terrain_RenderLODTile | 1 |
| 00451110 | Terrain_InitializeUVRotationTables | 4 |
| 004697e0 | Terrain_InitRenderTables | 4 |
| 00469bd0 | Terrain_ComputeTextMetrics | 1 |
| 0046a4c0 | Terrain_GetLODFlag | 25 |
| 0046ac90 | Terrain_RenderOrchestrator | 1 |
| 0046dc10 | Terrain_GenerateVertices | 2 |
| 0046e040 | Terrain_CheckTileVisibility | 1 |
| 0046e0f0 | Terrain_GenerateTriangles | 1 |
| 0046e870 | Terrain_CheckTriangleVisibility | 12 |
| 0046ebd0 | Terrain_TransformVertex | 12 |
| 0046efe0 | Terrain_PostRenderCleanup | 1 |
| 0046f6f0 | Terrain_EmitTriangle | 8 |
| 0046fb40 | Triangle_CreateWithRotatedUVs | 4 |
| 0046fc30 | Terrain_ProcessSpecialTile | 1 |
| 00470930 | Terrain_SubmitSpecialTileCmd12 | 1 |
| 00470a50 | Terrain_SubmitSpecialTileCmd0C | 1 |
| 00470b60 | Terrain_SubmitSpecialTileCmd13 | 1 |
| 00470d20 | Terrain_SubmitSpecialTileCmd09 | 1 |
| 00470e30 | Terrain_SubmitSpecialTileCmd0A | 1 |
| 00470f40 | Terrain_SubmitSpecialTileCmd0B | 1 |
| 00471040 | Terrain_RenderSpecialTileModel | 1 |
| 00472330 | Terrain_RenderSpecialTileBuildingShape | 1 |
| 004737c0 | Terrain_RenderSpecialTilePathLines | 1 |
| 00473a70 | Terrain_FinalizeRender | 1 |
| 00473bd0 | Terrain_RenderWithMatrix | 3 |
| 00474d60 | Terrain_RenderSpecialTileToDepthBucket | 2 |
| 00474e80 | Terrain_RenderSpecialTileCoastMesh | 1 |
| 00475530 | Terrain_RenderSpecialCells | 1 |
| 00475830 | Terrain_EmitOverlayEffects | 1 |
| 00487e30 | Render_Process3DModels | 1 |
| 004887a0 | Terrain_GetQuadCornerInfo | 1 |
| 00489360 | Terrain_RenderTile_Textured | 2 |
| 00489ea0 | Terrain_RenderTile_Flat | 3 |
| 0048aa00 | Terrain_RenderTile_Water | 1 |
| 0048bda0 | Tick_UpdateTerrain | 3 |
| 004e8300 | Terrain_QueueFlattenArea | 32 |
| 004e8450 | Terrain_RecalculateNormals | 6 |
| 004e8d60 | Terrain_QueueFlatten | 1 |
| 004e8e50 | Terrain_InterpolateHeight | 239 |
| 004e9e90 | Terrain_IsPointAccessible | 22 |
| 004e9fe0 | Cell_UpdateFlags | 4 |
| 004ea2e0 | Terrain_ModifyHeight | 4 |
| 004ea480 | Terrain_MarkCellOccupancy | 11 |
| 004eb260 | Cell_GetBuildingAltitude | 2 |
| 004f5ed0 | Terrain_PlaceAndFlatten | 8 |
| 0050ec30 | Terrain_GetCellWalkability | 33 |

## Tick (20 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 004198f0 | Tick_UpdatePopulation | 3 |
| 004552e0 | Tick_ResetSinglePlayerLevel | 4 |
| 004554b0 | Tick_LoadReplayData | 4 |
| 00455900 | Tick_LoadReplayData | 3 |
| 00455cc0 | Tick_ExecuteReplayScript | 3 |
| 00456500 | Tick_UpdateSinglePlayer | 1 |
| 00456ab0 | Tick_FinishCameraScript | 1 |
| 00456c00 | Tick_ProcessCameraScript | 1 |
| 00456fd0 | Tick_UpdateCameraMotion | 1 |
| 00469320 | Tick_UpdateTutorial | 1 |
| 0048bf10 | Tick_UpdateWater | 13 |
| 004a6f60 | Tick_ProcessPendingActions | 3 |
| 004a7550 | Tick_UpdateObjects | 9 |
| 004a76b0 | Tick_ProcessNetworkMessages | 3 |
| 004a7ac0 | Tick_UpdateGameTime | 4 |
| 004aeac0 | Tick_UpdateMana | 3 |
| 004b1bc0 | Tick_GatherManaFromFollowers | 1 |
| 004b2890 | Tick_SpreadManaToTerrain | 1 |
| 004b3000 | Tick_AutoSaveGameState | 1 |
| 004b3230 | Tick_UpdateSelectedObjectSound | 1 |
| 004e4f70 | Tick_SendNetworkMessage | 1 |
| 004e51c0 | Tick_OpenSyncLog | 1 |
| 004e53c0 | Tick_SendGameTick | 1 |
| 004e55d0 | Tick_SyncNetworkState | 1 |
| 004e6e60 | Tick_ReadNetworkPacket | 1 |

## Timer (4 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00426350 | Timer_GetTimeMs | 1 |
| 004263f0 | Timer_GetElapsed | 11 |
| 0042a870 | Timer_GetTickCount | 1 |
| 00513640 | timeGetTime | 2 |
| 00513650 | Thread_Sleep | 1 |

## Tribe (5 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00421bd0 | Tribe_InitAllStats | 1 |
| 00421c00 | Tribe_InitializeDefaultStats | 1 |
| 00421d70 | Tribe_InitializeSingleTribeStats | 1 |
| 00426f70 | Tribe_RespawnShaman | 2 |
| 004b4400 | Tribe_GetShaman | 23 |
| 004b5000 | Tribe_TrackKill | 4 |
| 004cd3a0 | Tribe_KillAllUnits | 3 |

## UI (23 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00403370 | UI_RenderChatMessages | 1 |
| 004066e0 | UI_RenderPanelAnimations | 2 |
| 00406140 | UI_RenderBorderedFrame | 27 |
| 00419920 | UI_RenderGameTimer | 1 |
| 00456e40 | UI_RenderPanelText | 1 |
| 00492390 | UI_RenderGamePanel | 2 |
| 00492df0 | GameState_ToggleOption | 19 |
| 00492e30 | UI_RenderObjectiveDisplay | 1 |
| 00493350 | UI_RenderResourceDisplay | 1 |
| 00493560 | UI_RenderStatusText | 1 |
| 004937f0 | UI_RenderBuildingInfo | 1 |
| 00494280 | UI_ClearScreenBuffer | 2 |
| 00494430 | UI_ProcessSpellButtons | 1 |
| 00494d90 | UI_RenderInfoPanel | 1 |
| 004ae5b0 | UI_RenderNetworkState | 8 |
| 004ae700 | UI_RenderMultiplayerStatus | 2 |
| 004d2780 | UI_RenderCompassPanel | 1 |
| 004d2a50 | UI_ProcessSoundSlider | 1 |
| 0048bd00 | HUD_SetMessage | 27 |
| 0048ef90 | UI_RenderVersionOverlay | 4 |
| 004e3ca0 | UI_LockNetworkState | 1 |
| 004e6210 | UI_SendMultiplayerPing | 1 |
| 004e6640 | UI_RenderNetworkDebugInfo | 1 |
| 004f04a0 | UI_RenderTimedLabel | 1 |

## UIWidget (5 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00544a80 | UIWidget_DispatchEvent | 23 |
| 00544ba0 | UIWidget_InsertChildBefore | 26 |
| 00544bc0 | UIWidget_SetChild | 26 |
| 00544c20 | UIWidget_DetachAndFree | 23 |
| 00544cb0 | UIWidget_FindChildById | 26 |

## Vehicle (18 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00497a10 | Vehicle_Init | 1 |
| 00497bd0 | Vehicle_SetState | 2 |
| 00497fe0 | Vehicle_Update | 1 |
| 00498400 | Vehicle_PlaceOnTerrain | 4 |
| 00498510 | Vehicle_ProcessMovement | 1 |
| 004986e0 | Vehicle_CheckArrival | 2 |
| 00498780 | Vehicle_UpdateBoatTravel | 1 |
| 00498a30 | Vehicle_UpdateBalloonTravel | 1 |
| 00498ce0 | Vehicle_NavigateToWaypoint | 1 |
| 00498f70 | Vehicle_DisembarkPassengers | 1 |
| 00499100 | Vehicle_UpdateSinking | 1 |
| 00499210 | Vehicle_UpdateDestruction | 1 |
| 0049b440 | Vehicle_UpdateSubmerge | 1 |
| 0049b5f0 | Vehicle_UpdateRotation | 2 |
| 0049b6f0 | Vehicle_UpdatePassengerAnimations | 3 |
| 004d6060 | Vehicle_SteerAndMove | 6 |
| 004d9f50 | Vehicle_ProcessMovement | 1 |

## Video / Crypto (2 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0041f650 | Video_PlayFMVSequence | 6 |
| 0041f960 | Crypto_XORDecryptBuffer | 2 |

## Water (6 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 00451b20 | Water_ResetAndUpdate | 6 |
| 00486d60 | Water_Initialize | 4 |
| 0048e210 | Water_AnimateMesh | 1 |
| 0048e730 | Water_SetupMesh | 3 |
| 0048e990 | Water_UpdateWavePhase | 1 |
| 004a75f0 | Water_RenderObjects | 2 |

## Misc Game (22 functions)

| Address | Name | XRefs |
|---------|------|-------|
| 0040b530 | DataBuf_Destroy | 20 |
| 0041d030 | String_WideToNarrow | 16 |
| 00419b40 | CRT_StreamDestroy | 18 |
| 0041f300 | Cursor_SubmitRenderCommand | 2 |
| 00442ff0 | Chat_FormatMessages | 3 |
| 00469dc0 | Chat_RecordMessage | 8 |
| 00486ff0 | set_topmap_onoff | 6 |
| 004ba1b0 | Error_ShowMessageBox | 145 |
| 00514040 | Stream_Destroy | 27 |
| 00511300 | CRT_VirtualDispatch | 16 |
| 00511310 | Resource_Alloc | 83 |
| 00511330 | Resource_Free | 101 |
| 005124d0 | Runtime_RegisterExitHandler | 1 |
| 0052bae0 | Runtime_RegisterExitHandler_B | 1 |
| 00512f60 | get_main_window_handle | 13 |
| 00512f80 | Win32_ProcessMessages | 2 |
| 0052eae0 | String_Copy | 4 |
| 0052eb40 | String_Append | 1 |
| 0052eb60 | String_ToInt | 2 |
| 0052eb70 | Char_IsDigit | 2 |
| 0052eb90 | String_Format | 1 |
| 0052ebb0 | String_FormatN | 1 |
| 005321f0 | Resource_AllocZeroed | 8 |
| 00535290 | File_FindClose | 2 |
| 00537f70 | File_OpenAlt | 1 |
| 00538670 | Heap_New | 1 |
| 0053ba10 | Thread_LeaveCritical | 11 |
| 00530730 | Atomic_Increment_A | 1 |
| 00534ea0 | Atomic_Increment_B | 1 |
| 00563af0 | Atomic_Increment | 9 |
| 0052bc00 | Sync_LeaveCritical | 4 |
| 0052d860 | Thread_ReleaseSemaphore | 19 |
| 0052d8c0 | Thread_ReleaseSemaphore | 9 |
| 0052bb40 | Thread_DeleteCritical | 1 |
| 00513d70 | System_GetVolumeInfo | 1 |

## Sync Signals (misc low-level)

| Address | Name | XRefs |
|---------|------|-------|
| 00512de6 | Sync_Signal_2de6 | 1 |
| 0052c7bc | Thread_Signal | 2 |
| 0052c894 | RenderCmd_SignalRead | 2 |
| 0052cd0e | Sync_Signal_cd0e | 1 |
| 0052d2e7 | Sync_Signal_d2e7 | 1 |
| 0052d2f9 | Sync_Signal_d2f9 | 1 |
| 0052dd1d | Sync_Signal_dd1d | 1 |
| 0052e03d | Sync_Signal_e03d | 1 |
| 0052e0e0 | Sync_Signal_e0e0 | 1 |
| 0052e171 | Sync_Signal_e171 | 1 |
| 0052e210 | Sync_Signal_e210 | 1 |
| 0052e274 | Sync_Signal_e274 | 1 |
| 0052e633 | Sync_Cleanup_e633 | 1 |

## Misc Named (thunks, stubs)

| Address | Name | XRefs |
|---------|------|-------|
| 00440360 | this_00440360 | 2 |
| 004abd60 | thunk_FUN_00410ff0 | 1 |
| 004e3d00 | param_1_004e3d00 | 1 |
| 004e3d80 | this_004e3d80 | 2 |
| 004e3e30 | this_004e3e30 | 2 |
| 0053a7f0 | thunk_FUN_0053a950 | 1 |
| 0053a9b0 | fptc_0053a9b0 | 1 |
| 00513b30 | thunk_FUN_0052e960 | 1 |
| 004baa0d | GameLoop_ThunkCleanup | 1 |
| 00537240 | CRT_BufferedRead | 39 |
| 005393a0 | Sound_InitSource | 19 |
| 005490d0 | lpStartAddress_005490d0 | 1 |

---

## CRT / Runtime Functions (~275)

Standard C runtime library functions (MSVC 6.0) included in the binary.
These are not game-specific and are listed here for completeness.

| Address | Name | XRefs |
|---------|------|-------|
| 00545fc0 | _swprintf | 142 |
| 00546050 | _sprintf | 140 |
| 005460d0 | CRT_Free | 215 |
| 005461b0 | ___CxxFrameHandler | 178 |
| 005464d0 | operator_new | 124 |
| 005464e0 | _wcsncpy | 7 |
| 00546530 | String_WideLength | 22 |
| 00546590 | _wcscpy | 229 |
| 005467d0 | __ftol | 80 |
| 00546800 | _sscanf | 13 |
| 00546850 | _fclose | 13 |
| 005468f0 | _fread | 6 |
| 00546a80 | _fseek | 6 |
| 00546b60 | _ftell | 1 |
| 00546d80 | FID_conflict:__wfopen | 12 |
| 00546da0 | _fwrite | 3 |
| 00547000 | _atexit | 15 |
| 00547330 | _iswctype | 13 |
| 005473b0 | _fscanf | 2 |
| 00547420 | _free | 145 |
| 00547490 | _malloc | 58 |
| 00547590 | _fprintf | 7 |
| 00547640 | _bsearch | 2 |
| 00547780 | _toupper | 12 |
| 005478e0 | _atol | 5 |
| 00547990 | _atoi | 5 |
| 00547a10 | _exit | 4 |
| 00547b50 | _clock | 5 |
| 00547f20 | _wcscmp | 32 |
| 00547f70 | _wcschr | 6 |
| 00548110 | _asctime | 3 |
| 00548220 | _gmtime | 7 |
| 00548380 | _time | 3 |
| 00548490 | entry | 2 |
| 00548940 | _strncpy | 12 |
| 00548f90 | _calloc | 16 |
| 00549040 | __beginthread | 5 |
| 00549190 | __endthread | 5 |
| 0054b560 | __getptd | 20 |
| 0054be70 | __input | 2 |
| 0054cc00 | __lock | 33 |
| 0054cc70 | CRT_UnlockSection | 49 |
| 0054d3f0 | __dosmaperr | 9 |
| 0054d470 | CRT_GetErrnoPtr | 46 |
| 0054d480 | CRT_GetDosErrnoPtr | 21 |
| 0054dd00 | _realloc | 3 |
| 0054e400 | __isctype | 23 |
| 00550940 | __setmbcp | 1 |
| 00550f80 | __mbsnbcpy | 5 |
| 00551360 | terminate | 10 |
| 005513e0 | _inconsistency | 9 |
| 00551710 | _mbtowc | 3 |
| 00551ad0 | _wctomb | 3 |
| 00556bf0 | ___getlocaleinfo | 64 |
| 00557020 | __strcmpi | 13 |
| 005570f0 | _localtime | 2 |
| 005572c0 | _wcstombs | 2 |

*(~220 additional CRT internals omitted for brevity — heap management, FP math, locale, etc.)*

## Win32 Imports (IME)

| Address | Name | XRefs |
|---------|------|-------|
| 005629ac | ImmReleaseContext | 12 |
| 005629b2 | ImmGetCompositionStringA | 6 |
| 005629b8 | ImmGetContext | 7 |
| 005629be | ImmSetOpenStatus | 2 |
| 005629c4 | ImmGetOpenStatus | 2 |
| 005629ca | ImmSetCompositionWindow | 1 |
| 005629d0 | ImmGetCandidateListA | 4 |
| 00562a00 | WinMain | 1 |
| 00563a48 | DirectInputCreateA | 2 |

---

## Renamed Data Labels

| Address | Original | New Name |
|---------|----------|----------|
| 0x005a0720 | DAT_005a0720 | g_VehicleTypeData |
| 0x005a3220 | DAT_005a3220 | SPELL_BURN_DAMAGE |
| 0x006868a8 | DAT_006868a8 | g_CameraTarget |
| 0x0059fb30 | DAT_0059fb30 | g_PersonAnimationTable |
| 0x0059f638 | DAT_0059f638 | g_AnimationFrameData |

---

## Summary

- **1,475 functions named** out of ~3,828 total (~38.5%)
- **~1,200 game functions** across 40+ subsystems
- **~275 CRT/runtime** functions (MSVC 6.0 standard library)
- **Zero unnamed functions with 16+ xrefs remaining**
- Key high-xref functions: `Render_Is16bppMode` (325), `Object_SetStateByType` (271), `Terrain_InterpolateHeight` (239), `_wcscpy` (229), `CRT_Free` (215), `Sound_Play` (211), `Object_Create` (188), `AI_EvaluateScriptValue` (184)
