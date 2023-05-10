import maya.cmds as cmds

curves = []
curveShapes = []

square = cmds.polyPlane(w=5, h=5, sx=1, sy=1, ax=(0,0,1), n='krrustyLight')[0]
edges = cmds.ls(square + ".e[:]", flatten=1)
for e in edges:
    cmds.select(e)
    curv = cmds.polyToCurve(form=2, degree=3, conformToSmoothMeshPreview=True)[0]
    curves.append(curv)
    curveShapes.append(cmds.listRelatives(curv, s=True)[0])
    cmds.delete(curv, constructionHistory = True)
    
aim = cmds.curve(d=1, p=[(0.0, 0, 0),(0.0, 0, 2.5),])
curves.append(aim)
curveShapes.append(cmds.listRelatives(aim, s=True)[0])
cmds.delete(square, constructionHistory = True)
cmds.rename(square, square + 'Plane')
light = cmds.group(em=True, name=square)
cmds.parent(curveShapes, light, s=True, r=True)
cmds.parent(square + 'Plane', light, s=True, r=True)
cmds.delete(curves)
cmds.setAttr(square + 'Plane' + '.overrideEnabled', 1)
cmds.setAttr(square + 'Plane' + '.hiddenInOutliner', 1)
cmds.setAttr(cmds.listRelatives(square + 'Plane', s=True)[0] + '.hiddenInOutliner', 1)
cmds.setAttr(cmds.listRelatives(square + 'Plane', s=True)[0] + '.visibility', 0)
cmds.addAttr(light, ln="krrustyLight", at="bool", ct="Krrust", dv=True, h=True)
cmds.addAttr(light, ln="lightPlane", dataType="string", ct="Krrust", h=True)
cmds.setAttr(light + '.lightPlane', square + 'Plane', type="string")
cmds.addAttr(light, ln='color', at="float3", ct="Krrust", uac=True)
cmds.addAttr(light, ln='red', at='float', parent='color', dv=1.0 )
cmds.addAttr(light, ln='green', at='float', parent='color', dv=1.0 )
cmds.addAttr(light, ln='blue', at='float', parent='color', dv=1.0 )
cmds.addAttr(light, ln='intensity', at='float', dv = 4.0)
cmds.select(light)


