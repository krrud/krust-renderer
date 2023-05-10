import maya.OpenMaya as OpenMaya
import maya.OpenMayaMPx as OpenMayaMPx
import sys


mtlPluginNodeName = "krustyMaterial"
mtlPluginNodeId = OpenMaya.MTypeId(0x00333)
lightPluginNodeName = "krustyLight"
lightPluginNodeId = OpenMaya.MTypeId(0x00000)


class krustyMaterial(OpenMayaMPx.MPxNode):

    def __init__(self):
        OpenMayaMPx.MPxNode.__init__(self)


def mtlNodeCreator():
    return OpenMayaMPx.asMPxPtr(krustyMaterial())


def mtlNodeInitializer():
    nAttr = OpenMaya.MFnNumericAttribute()
    kFloat = OpenMaya.MFnNumericData.kFloat

    # input attributes
    krustyMaterial.diffuse = nAttr.createColor('diffuse', 'diff')
    nAttr.setStorable(True)
    nAttr.setDefault(0.05, 0.7, 0.5)
    krustyMaterial.addAttribute(krustyMaterial.diffuse)
    
    krustyMaterial.diffuseWeight = nAttr.createColor('diffuseWeight', 'dwt')
    nAttr.setStorable(True)
    nAttr.setDefault(1.0, 1.0, 1.0)
    krustyMaterial.addAttribute(krustyMaterial.diffuseWeight)

    krustyMaterial.specular = nAttr.createColor('specular', 'spec')
    nAttr.setStorable(True)
    nAttr.setDefault(1.0, 1.0, 1.0)
    krustyMaterial.addAttribute(krustyMaterial.specular)

    krustyMaterial.specularWeight = nAttr.createColor('specularWeight', 'swt')
    nAttr.setStorable(True)
    nAttr.setDefault(1.0, 1.0, 1.0)
    krustyMaterial.addAttribute(krustyMaterial.specularWeight)

    krustyMaterial.roughness = nAttr.createColor('roughness', 'rough')
    nAttr.setStorable(True)
    nAttr.setDefault(0.4, 0.4, 0.4)
    krustyMaterial.addAttribute(krustyMaterial.roughness)

    krustyMaterial.ior = nAttr.create('IOR', 'ior', kFloat, 1.5)
    nAttr.setStorable(True)
    nAttr.setMin(1.01)
    nAttr.setMax(3)
    krustyMaterial.addAttribute(krustyMaterial.ior)

    krustyMaterial.metallic = nAttr.createColor('metallic', 'met')
    nAttr.setStorable(True)
    nAttr.setDefault(0.0, 0.0, 0.0)
    krustyMaterial.addAttribute(krustyMaterial.metallic)

    krustyMaterial.refraction = nAttr.createColor('refraction', 'refr')
    nAttr.setStorable(True)
    nAttr.setDefault(0.0, 0.0, 0.0)
    krustyMaterial.addAttribute(krustyMaterial.refraction)

    krustyMaterial.emission = nAttr.createColor('emission', 'e')
    nAttr.setStorable(True)
    nAttr.setDefault(0.0, 0.0, 0.0)
    krustyMaterial.addAttribute(krustyMaterial.emission)

    krustyMaterial.bump = nAttr.createColor('bump', 'b')
    nAttr.setStorable(True)
    nAttr.setDefault(0.0, 0.0, 0.0)
    krustyMaterial.addAttribute(krustyMaterial.bump)

    krustyMaterial.normal = nAttr.createColor('normal', 'n')
    nAttr.setStorable(True)
    nAttr.setDefault(0.0, 0.0, 0.0)
    krustyMaterial.addAttribute(krustyMaterial.normal)

    krustyMaterial.bumpStrength = nAttr.create('bumpStrength', 'bs', kFloat, 1.0)
    nAttr.setStorable(True)
    krustyMaterial.addAttribute(krustyMaterial.bumpStrength)

    krustyMaterial.normalStrength = nAttr.create('normalStrength', 'ns', kFloat, 1.0)
    nAttr.setStorable(True)
    krustyMaterial.addAttribute(krustyMaterial.normalStrength)


##############################################################################################
##############################################################################################


class krustyLight(OpenMayaMPx.MPxNode):

    def __init__(self):
        OpenMayaMPx.MPxNode.__init__(self)


def lightNodeCreator():
    return OpenMayaMPx.asMPxPtr(krustyLight())


def lightNodeInitializer():
    numericAttributeFn = OpenMaya.MFnNumericAttribute()
    kFloat = OpenMaya.MFnNumericData.kFloat

    # input attributes
    krustyLight.color = numericAttributeFn.createColor('color', 'color')
    numericAttributeFn.setStorable(True)
    numericAttributeFn.setDefault(1.0, 1.0, 1.0)
    krustyLight.addAttribute(krustyLight.color)

    krustyLight.intensity = numericAttributeFn.create(
        'intensity', 'intensity', kFloat, 1.0)
    numericAttributeFn.setStorable(True)
    krustyLight.addAttribute(krustyLight.intensity)


all_nodes = [[mtlPluginNodeName, mtlPluginNodeId, mtlNodeCreator, mtlNodeInitializer],
             [lightPluginNodeName, lightPluginNodeId, lightNodeCreator, lightNodeInitializer], ]


def initializePlugin(mobject):
    mplugin = OpenMayaMPx.MFnPlugin(mobject, "", "", "Any")
    for node in all_nodes:
        try:
            mplugin.registerNode(node[0], node[1], node[2], node[3])
        except:
            sys.stderr.write("Failed to register command: %s" % node[0])
            raise


def uninitializePlugin(mobject):
    mplugin = OpenMayaMPx.MFnPlugin(mobject)
    for node in all_nodes:
        try:
            mplugin.deregisterNode(node[1])
        except:
            sys.stderr.write("Failed to unregister node: %s" % node[0])
            raise
