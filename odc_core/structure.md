# Structure of render engine core:

## PassTree
- PassName -> [PassName]

## Pass
- Name
- [PipelineName]
- [Attachment]

## Pipeline
- Name
- [InputBufferName]
- [BindGroupName]
- Shader
- Option<DepthOpts>

## Attachment
- TextureName
- TexturePart

## InputBuffer
- BufferName
- [Attribute]
- InputType
- Stride

## BindGroup
- Name
- [BufferBinding]
- [TextureBinding]
- [SamplerBinding]

## Shader
- URI
- VertexEntryPoint
- IndexEntryPoint

## DepthOpts
- TextureName

## Texture
- Name
- Format
- Size2d

## TexturePart
- Offset2d
- Size2d

## Buffer
- Name
- Size
- BufferType

## BufferBinding
- Id
- BufferName
- Size
- Offset

## TextureBinding
- Id
- TextureName
- TexturePart

## SamplerBinding
- Id
- SamplerType