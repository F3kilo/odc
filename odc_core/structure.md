# Structure of render engine core:

## PassGraph
- PassMap

## Pass
- Name
- [PipelineName]
- [Attachment]

## Pipeline
- Name
- [InputBufferName]
- [BindGroupName]
- Shader
- Option<Depth>

## Attachment
- TextureName
- TexturePart

## InputBuffer
- BufferName
- StepMode
- [Attribute]
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

## Depth
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
- SamplerName

## Sampler
- SamplerType