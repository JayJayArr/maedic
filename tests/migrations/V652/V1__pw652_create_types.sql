/****** Object:  UserDefinedDataType [dbo].[MICBINID]    Script Date: 03.02.2026 11:21:00 ******/
CREATE TYPE [dbo].[MICBINID] FROM [varbinary](18) NOT NULL

/****** Object:  UserDefinedDataType [dbo].[MICCARD]    Script Date: 03.02.2026 11:21:00 ******/
CREATE TYPE [dbo].[MICCARD] FROM [nvarchar](32) NOT NULL

/****** Object:  UserDefinedDataType [dbo].[MICDESCRP]    Script Date: 03.02.2026 11:21:00 ******/
CREATE TYPE [dbo].[MICDESCRP] FROM [nvarchar](40) NOT NULL

/****** Object:  UserDefinedDataType [dbo].[MICHWDESCRP]    Script Date: 03.02.2026 11:21:00 ******/
CREATE TYPE [dbo].[MICHWDESCRP] FROM [nvarchar](80) NULL

/****** Object:  UserDefinedDataType [dbo].[MICID]    Script Date: 03.02.2026 11:21:00 ******/
CREATE TYPE [dbo].[MICID] FROM [nvarchar](64) NOT NULL

/****** Object:  UserDefinedDataType [dbo].[MICIDSUPER]    Script Date: 03.02.2026 11:21:00 ******/
CREATE TYPE [dbo].[MICIDSUPER] FROM [varbinary](128) NULL

/****** Object:  UserDefinedDataType [dbo].[SURVFILENAME]    Script Date: 03.02.2026 11:21:00 ******/
CREATE TYPE [dbo].[SURVFILENAME] FROM [nvarchar](260) NULL
