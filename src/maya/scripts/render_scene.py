import maya.cmds as cmds
import maya.OpenMaya as OpenMaya
import maya.mel as mel
import json
import math 
import subprocess
import os


def to_linear(sRGB):
    if sRGB <= 0.04045:
        linear = sRGB / 12.92
    else:
        linear = ((sRGB + 0.055) / 1.055) ** 2.4
    return linear


selection_cache = cmds.ls(selection=True)
processed = []
def get_spheres():
    sphere_objects = []
    for obj in cmds.ls(typ="transform"):
        obj_nodes = cmds.listHistory(obj)
        for connected_node in obj_nodes:
            if cmds.nodeType(connected_node) == 'polySphere' and  cmds.getAttr('%s.visibility' % obj) == True:
                radius = cmds.getAttr('%s.radius' % connected_node)
                loc = cmds.xform(obj, q=1, ws=1, rp=1)
                try:
                    mat = cmds.getAttr(obj + '.krrustyMaterial')
                except:
                    mat = 'defaultMaterial'
                s = {"radius": radius, "location": loc, "material": mat}
                sphere_objects.append(s)
                processed.append(obj)
    return sphere_objects

def get_meshes():
    mesh_objects = []
    for obj in cmds.ls(typ="transform"):
        if obj not in processed:
            obj_nodes = cmds.listHistory(obj)
            for connected_node in obj_nodes:
                if cmds.nodeType(connected_node) == 'mesh' and  cmds.getAttr('%s.visibility' % obj) == True:
                    vtx_p, vtx_n, vtx_uv = get_vtx(obj)
                    try:
                        mat = cmds.getAttr(obj + '.krrustyMaterial')
                    except:
                        mat = 'defaultMaterial'
                    mesh = {
                        "name": obj,
                        "vertices": vtx_p,
                        "normals": vtx_n,
                        "uvs": vtx_uv,
                        "material": mat
                    }
                    mesh_objects.append(mesh)
    return mesh_objects
 
def get_vtx_old(node):
    shape = cmds.listRelatives(node, s=True)
    cmds.select(shape)
    selList = OpenMaya.MSelectionList()
    OpenMaya.MGlobal.getActiveSelectionList(selList)
    dagPath = OpenMaya.MDagPath()
    selList.getDagPath(0, dagPath)
    meshFn = OpenMaya.MFnMesh(dagPath)

    cursor, out_vtx_p, out_vtx_n, out_vtx_uv = 0, [], [], []
    meshVtxCount = OpenMaya.MIntArray()
    meshVtxArray = OpenMaya.MIntArray() 

    u_array = OpenMaya.MFloatArray()
    v_array = OpenMaya.MFloatArray()  
    meshFn.getVertices(meshVtxCount, meshVtxArray)
    meshFn.getUVs(u_array, v_array)

    for i, count in enumerate(meshVtxCount):
        faceVtx = meshVtxArray[cursor: cursor + count]
        # uv = [list(u_array[i: i + count]), list(v_array[i: i + count])]
        # out_vtx_uv.append(uv)
        cursor += count
        p, n, uv = [], [], []
        for id in faceVtx:
            vtxPosition = OpenMaya.MPoint()
            vtxNormal = OpenMaya.MVector()
            u = OpenMaya.MScriptUtil()
            v = OpenMaya.MScriptUtil()
            uPtr = u.asFloatPtr()
            vPtr = v.asFloatPtr()
            meshFn.getPoint(id, vtxPosition, 4)
            meshFn.getVertexNormal(id, True, vtxNormal, 4)
            meshFn.getUV(id, uPtr, vPtr)
            u_val = u.getFloat(uPtr)
            v_val = v.getFloat(vPtr)
            p.append([vtxPosition[0], vtxPosition[1], vtxPosition[2]])
            n.append([vtxNormal[0], vtxNormal[1], vtxNormal[2]])
            if [u_val, v_val] not in uv:
                uv.append([u_val, v_val])
        out_vtx_p.append(p)
        out_vtx_n.append(n)

    return out_vtx_p, out_vtx_n, out_vtx_uv


def get_vtx(node):
    shape = cmds.listRelatives(node, s=True)
    cmds.select(shape)
    selList = OpenMaya.MSelectionList()
    OpenMaya.MGlobal.getActiveSelectionList(selList)
    dagPath = OpenMaya.MDagPath()
    selList.getDagPath(0, dagPath)
    meshFn = OpenMaya.MFnMesh(dagPath)

    out_vtx_p, out_vtx_n, out_vtx_uv = [], [], []
    meshVtxCount = OpenMaya.MIntArray()
    meshVtxArray = OpenMaya.MIntArray() 

    u_array = OpenMaya.MFloatArray()
    v_array = OpenMaya.MFloatArray()  
    meshFn.getVertices(meshVtxCount, meshVtxArray)
    meshFn.getUVs(u_array, v_array)

    uvCounts = OpenMaya.MIntArray()
    uvIds = OpenMaya.MIntArray()
    meshFn.getAssignedUVs(uvCounts, uvIds)

    uvIndex = 0
    for i, count in enumerate(meshVtxCount):
        faceVtx = meshVtxArray[uvIndex: uvIndex + count]
        p, n, uv = [], [], []
        for j, id in enumerate(faceVtx):
            vtxPosition = OpenMaya.MPoint()
            vtxNormal = OpenMaya.MVector()
            meshFn.getPoint(id, vtxPosition, OpenMaya.MSpace.kWorld)
            meshFn.getVertexNormal(id, True, vtxNormal, OpenMaya.MSpace.kWorld)
            u_val = u_array[uvIds[uvIndex + j]]
            v_val = v_array[uvIds[uvIndex + j]]
            p.append([vtxPosition[0], vtxPosition[1], vtxPosition[2]])
            n.append([vtxNormal[0], vtxNormal[1], vtxNormal[2]])
            uv.append([u_val, v_val])
        out_vtx_p.append(p)
        out_vtx_n.append(n)
        out_vtx_uv.append(uv)
        uvIndex += count

    return out_vtx_p, out_vtx_n, out_vtx_uv


def get_materials():
    all_materials = []
    materials = cmds.ls(typ='krrustyMaterial')
    for m in materials:
        # swatches        
        diffuse = cmds.getAttr(m+'.diffuse')[0]
        # diffuse = [to_linear(diffuse[0]), to_linear(diffuse[1]), to_linear(diffuse[2])]
        diffuse_weight = cmds.getAttr(m+'.diffuseWeight')[0]
        specular = cmds.getAttr(m+'.specular')[0]
        specular_weight = cmds.getAttr(m+'.specularWeight')[0]
        roughness = cmds.getAttr(m+'.roughness')[0]
        metallic = cmds.getAttr(m+'.metallic')[0]
        refraction = cmds.getAttr(m+'.refraction')[0]
        emission = cmds.getAttr(m+'.emission')[0]
        bump = cmds.getAttr(m+'.bump')[0]
        bump_strength = cmds.getAttr(m+'.bumpStrength')
        normal_strength = cmds.getAttr(m+'.normalStrength')
        ior = cmds.getAttr(m+'.ior')
    
        # textures
        diffuse_tex = ''
        dt = cmds.listConnections(m+'.diffuse', type='file')
        if dt:
            diffuse_tex = cmds.getAttr(dt[0] + '.fileTextureName')

        diffuse_weight_tex = ''
        dwt = cmds.listConnections(m+'.diffuseWeight', type='file')
        if dwt:
            diffuse_weight_tex = cmds.getAttr(dwt[0] + '.fileTextureName')

        specular_tex = ''
        st = cmds.listConnections(m+'.specular', type='file')
        if st:
            specular_tex = cmds.getAttr(st[0] + '.fileTextureName')

        specular_weight_tex = ''
        swt = cmds.listConnections(m+'.specularWeight', type='file')
        if swt:
            specular_weight_tex = cmds.getAttr(swt[0] + '.fileTextureName')

        roughness_tex = ''
        rt = cmds.listConnections(m+'.roughness', type='file')
        if rt:
            roughness_tex = cmds.getAttr(rt[0] + '.fileTextureName')
            
        metallic_tex = ''
        mt = cmds.listConnections(m+'.metallic', type='file')
        if mt:
            metallic_tex = cmds.getAttr(mt[0] + '.fileTextureName')

        refraction_tex = ''
        rft = cmds.listConnections(m+'.refraction', type='file')
        if rft:
            refraction_tex = cmds.getAttr(rft[0] + '.fileTextureName')

        emission_tex = ''
        et = cmds.listConnections(m+'.emission', type='file')
        if et:
            emission_tex = cmds.getAttr(et[0] + '.fileTextureName')

        bump_tex = ''
        bt = cmds.listConnections(m+'.bump', type='file')
        if bt:
            bump_tex = cmds.getAttr(bt[0] + '.fileTextureName')

        normal_tex = ''
        nt = cmds.listConnections(m+'.normal', type='file')
        if nt:
            normal_tex = cmds.getAttr(nt[0] + '.fileTextureName')

        mat = {
            "name": m,
            "diffuse": diffuse,
            "diffuse_tex": diffuse_tex,
            "diffuse_weight": diffuse_weight,
            "diffuse_weight_tex": diffuse_weight_tex,
            "specular": specular,
            "specular_tex": specular_tex,
            "specular_weight": specular_weight,
            "specular_weight_tex": specular_weight_tex,
            "roughness": roughness,
            "roughness_tex": roughness_tex,
            "ior": ior,
            "metallic": metallic,
            "metallic_tex": metallic_tex,
            "refraction": refraction,
            "refraction_tex": refraction_tex,
            "emission": emission,
            "emission_tex": emission_tex,
            "bump": bump,
            "bump_tex": bump_tex,
            "bump_strength": bump_strength,
            "normal_tex": normal_tex,
            "normal_strength": normal_strength,
        }
        all_materials.append(mat)
    return all_materials

def get_quad_lights():
    all_lights = []
    lights = cmds.ls('*.krrustyLight', o=True)
    for light in lights:
        color = cmds.getAttr(light + '.color')[0]
        intensity = cmds.getAttr(light + '.intensity')
        light_plane = cmds.getAttr(light + '.lightPlane')
        points = get_vtx(light_plane)[0]
        new_light = {
            "name": light,
            "color": color,
            "intensity": intensity,
            "points": points,
        }
        all_lights.append(new_light)
    return all_lights

def get_dir_lights():
    dir_lights = []
    lights = cmds.ls(type='directionalLight')
    for light in lights:
        color = cmds.getAttr(light + '.color')[0]
        intensity = cmds.getAttr(light + '.intensity')
        softness = cmds.getAttr(light + '.aiAngle')
        x = cmds.xform(cmds.listRelatives(light, parent=True)[0], query=True, ws=True, m=True)
        direction = [x[8], x[9], x[10]]
        new_light = {
            "name": light,
            "direction": direction,
            "color": color,
            "intensity": intensity,
            "softness": softness
        }
        dir_lights.append(new_light)
    return dir_lights

spheres = get_spheres()
meshes = get_meshes()
materials = get_materials()
quad_lights = get_quad_lights()
dir_lights = get_dir_lights()
lights = {"quad": quad_lights, "dir": dir_lights}
cam = "camera1"
cam_shape = cmds.listRelatives(cam, shapes=True)[0]
cam_aim = "camera1_aim"
camera_focus = "camera1_focus"
aperature = cmds.getAttr(cam + '.hfa')
focal_length = cmds.getAttr(cam + '.focalLength')
fov = (aperature * 0.5) / (focal_length * 0.03937)
fov = math.tan(fov) * 57.29578
aspect_ratio = float(cmds.getAttr("defaultResolution.width")) / float(cmds.getAttr("defaultResolution.height"))
width = 1920
height = int(width / aspect_ratio)
aperature = cmds.getAttr(cam + '.aperature') / 10

scene_data = {
    "scene": {
        "meshes": meshes,
        "spheres": spheres,
        "materials": materials,
        "lights": lights,
        "mesh_count": len(meshes),
        "sphere_count": len(spheres),
        "material_count": len(materials),
        "quad_light_count": len(quad_lights),
        "dir_light_count": len(dir_lights)
    },
    "settings": {
        "progressive": 1,
        "width": width,
        "height": height, 
        "aspect_ratio": aspect_ratio,
        "spp": 1024,
        "depth": 32,
        "aperature": aperature,
        "fov": fov,
        "camera_origin": [
            cmds.getAttr(cam + '.tx'),
            cmds.getAttr(cam + '.ty'),
            cmds.getAttr(cam + '.tz'),
        ],
        "camera_aim": [
            cmds.getAttr(cam_aim + '.tx'),
            cmds.getAttr(cam_aim + '.ty'),
            cmds.getAttr(cam_aim + '.tz'),
        ],
        "camera_focus": [
            cmds.getAttr(camera_focus + '.tx'),
            cmds.getAttr(camera_focus + '.ty'),
            cmds.getAttr(camera_focus + '.tz'),
        ],
        "output_file": "G:/krrust_output_new2.exr"
    }
}

data_location = r"G:\rust_projects\krrust\target\debug\render_data.json"
data_json = json.dumps(scene_data, indent=4)
with open(data_location, "w") as outfile:
    outfile.write(data_json)

cmds.select(selection_cache)
exe_path = r"G:\rust_projects\krrust\target\debug"
os.chdir(exe_path)
render = subprocess.Popen("krrust.exe")#, shell=True
# render.terminate()

