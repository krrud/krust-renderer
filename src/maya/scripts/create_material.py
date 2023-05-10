import maya.cmds as cmds

meshes = cmds.ls(selection=True, typ="mesh", dag=1, ni=1)
if meshes:
    material = cmds.createNode("krrustyMaterial")
    for mesh in meshes:
        if not cmds.attributeQuery('krrustyMaterial', node=mesh, exists=True):
            cmds.addAttr(mesh, ln="krrustyMaterial", dataType="string", ct="Krrust")
        cmds.setAttr(mesh + '.krrustyMaterial', material, type="string")
else:
    cmds.warning("Invalid Selection")