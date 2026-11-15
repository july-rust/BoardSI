-- F_002: BoardSI .bsi schema v1 (SQLite >= 3.37). Contract, not a Writer/Reader.
-- Open a new EMPTY staging database; never apply this script to an existing .bsi.
PRAGMA encoding = 'UTF-8';
PRAGMA page_size = 4096;
PRAGMA auto_vacuum = NONE;
PRAGMA journal_mode = DELETE;
PRAGMA trusted_schema = OFF;
PRAGMA foreign_keys = ON;
PRAGMA application_id = 0x42534931;
PRAGMA user_version = 1;

CREATE TABLE file_meta (
  singleton INTEGER PRIMARY KEY CHECK(singleton=1),
  file_uuid TEXT NOT NULL CHECK(length(file_uuid)=36),
  format_major INTEGER NOT NULL CHECK(format_major=1),
  format_minor INTEGER NOT NULL CHECK(format_minor>=0),
  min_reader_major INTEGER NOT NULL CHECK(min_reader_major>=1),
  min_reader_minor INTEGER NOT NULL CHECK(min_reader_minor>=0),
  board_kind TEXT NOT NULL CHECK(board_kind IN ('Pcb','BgaSubstrate')),
  producer TEXT NOT NULL CHECK(length(producer)>0),
  producer_version TEXT NOT NULL,
  created_utc TEXT NOT NULL,
  CHECK(min_reader_major<format_major OR
    (min_reader_major=format_major AND min_reader_minor<=format_minor))
) STRICT;

CREATE TABLE features (
  name TEXT NOT NULL,
  feature_version INTEGER NOT NULL CHECK(feature_version>=1),
  required INTEGER NOT NULL CHECK(required IN (0,1)),
  opaque_payload BLOB,
  PRIMARY KEY(name,feature_version),
  CHECK(length(name)>0)
) STRICT, WITHOUT ROWID;

CREATE TABLE source_identity (
  id INTEGER PRIMARY KEY,
  source_format TEXT NOT NULL CHECK(length(source_format)>0),
  format_other TEXT,
  format_version TEXT NOT NULL,
  importer_version TEXT NOT NULL,
  producer TEXT NOT NULL,
  producer_version TEXT NOT NULL,
  CHECK((source_format='Other')=(format_other IS NOT NULL)),
  CHECK(format_other IS NULL OR length(format_other)>0)
) STRICT;
CREATE TABLE source_files (
  source_id INTEGER NOT NULL REFERENCES source_identity(id) ON DELETE RESTRICT,
  ordinal INTEGER NOT NULL CHECK(ordinal>=0),
  logical_path TEXT NOT NULL CHECK(length(logical_path)>0),
  sha256 BLOB NOT NULL CHECK(length(sha256)=32),
  sha_verified INTEGER NOT NULL CHECK(sha_verified IN (0,1)),
  PRIMARY KEY(source_id,ordinal),
  UNIQUE(source_id,logical_path)
) STRICT, WITHOUT ROWID;
CREATE TABLE source_conditions (
  source_id INTEGER NOT NULL REFERENCES source_identity(id) ON DELETE RESTRICT,
  ordinal INTEGER NOT NULL CHECK(ordinal>=0),
  condition_key TEXT NOT NULL CHECK(length(condition_key)>0),
  condition_value TEXT NOT NULL,
  PRIMARY KEY(source_id,ordinal),
  UNIQUE(source_id,condition_key)
) STRICT, WITHOUT ROWID;
CREATE TABLE import_capability (
  source_id INTEGER NOT NULL REFERENCES source_identity(id) ON DELETE RESTRICT,
  dimension TEXT NOT NULL CHECK(dimension IN ('G','N','P','S','R','M')),
  state TEXT NOT NULL CHECK(state IN ('Complete','Partial','Missing','Conflicting')),
  PRIMARY KEY(source_id,dimension)
) STRICT, WITHOUT ROWID;
CREATE TABLE capability_coverage (
  source_id INTEGER NOT NULL REFERENCES source_identity(id) ON DELETE RESTRICT,
  ordinal INTEGER NOT NULL CHECK(ordinal>=0),
  covered_kind TEXT NOT NULL,
  PRIMARY KEY(source_id,ordinal)
) STRICT, WITHOUT ROWID;

-- Semantic identity: (full source_identity, kind, source_object, part).
-- INTEGER keys are LOCAL FKs only, never exported as ObjectId.
CREATE TABLE objects (
  id INTEGER PRIMARY KEY,
  source_id INTEGER NOT NULL REFERENCES source_identity(id) ON DELETE RESTRICT,
  kind TEXT NOT NULL CHECK(kind IN
    ('Frame','Layer','Material','Net','Component','Pin','Trace','Arc',
     'Area','Cutout','Padstack','Pad','Via','Drill','Outline','Node','Edge')),
  source_object TEXT NOT NULL,
  part TEXT NOT NULL,
  source_object_type TEXT NOT NULL,
  UNIQUE(source_id,kind,source_object,part)
) STRICT;
CREATE INDEX objects_source_kind ON objects(source_id,kind);

CREATE TABLE conversion_history (
  object_id INTEGER NOT NULL REFERENCES objects(id) ON DELETE RESTRICT,
  ordinal INTEGER NOT NULL CHECK(ordinal>=0),
  operation TEXT NOT NULL CHECK(length(operation)>0),
  quality TEXT NOT NULL CHECK(quality IN ('Exact','Approximate','Lossy','Pending')),
  error_num_le BLOB CHECK(error_num_le IS NULL OR length(error_num_le)=16),
  error_den_le BLOB CHECK(error_den_le IS NULL OR length(error_den_le)=16),
  note TEXT NOT NULL,
  PRIMARY KEY(object_id,ordinal),
  CHECK((error_num_le IS NULL)=(error_den_le IS NULL))
) STRICT, WITHOUT ROWID;

CREATE TABLE coordinate_frames (
  object_id INTEGER PRIMARY KEY REFERENCES objects(id) ON DELETE RESTRICT,
  unit_label TEXT NOT NULL,
  metre_num_le BLOB NOT NULL CHECK(length(metre_num_le)=16),
  metre_den_le BLOB NOT NULL CHECK(length(metre_den_le)=16),
  origin_x_num_le BLOB NOT NULL CHECK(length(origin_x_num_le)=16),
  origin_x_den_le BLOB NOT NULL CHECK(length(origin_x_den_le)=16),
  origin_y_num_le BLOB NOT NULL CHECK(length(origin_y_num_le)=16),
  origin_y_den_le BLOB NOT NULL CHECK(length(origin_y_den_le)=16),
  orientation TEXT NOT NULL CHECK(orientation IN ('Orthogonal','PendingRotation')),
  quarter_turn INTEGER CHECK(quarter_turn BETWEEN 0 AND 3),
  pending_radians REAL,
  mirror_x INTEGER NOT NULL CHECK(mirror_x IN (0,1)),
  reverse_source_layer_order INTEGER NOT NULL CHECK(reverse_source_layer_order IN (0,1)),
  CHECK((orientation='Orthogonal' AND quarter_turn IS NOT NULL AND pending_radians IS NULL)
     OR (orientation='PendingRotation' AND quarter_turn IS NULL AND pending_radians IS NOT NULL))
) STRICT;

CREATE TABLE materials (
  object_id INTEGER PRIMARY KEY REFERENCES objects(id) ON DELETE RESTRICT,
  name TEXT NOT NULL
) STRICT;
CREATE TABLE material_properties (
  material_id INTEGER NOT NULL REFERENCES materials(object_id) ON DELETE RESTRICT,
  property_key TEXT NOT NULL CHECK(property_key IN ('Dk','Df','Conductivity','ValidBand')),
  status TEXT NOT NULL CHECK(status IN ('Missing','Assumed','Confirmed')),
  value_blob BLOB,
  document TEXT, revision TEXT, method TEXT, reference_frequency_hz REAL, temperature_c REAL,
  PRIMARY KEY(material_id,property_key),
  CHECK((status='Missing' AND value_blob IS NULL AND document IS NULL AND revision IS NULL AND method IS NULL)
      OR (status IN ('Assumed','Confirmed') AND value_blob IS NOT NULL AND document IS NOT NULL
          AND revision IS NOT NULL AND method IS NOT NULL))
) STRICT, WITHOUT ROWID;

CREATE TABLE layers (
  object_id INTEGER PRIMARY KEY REFERENCES objects(id) ON DELETE RESTRICT,
  layer_order INTEGER NOT NULL UNIQUE CHECK(layer_order>=0),
  name TEXT NOT NULL,
  layer_kind TEXT NOT NULL CHECK(layer_kind IN ('Copper','DielectricCore','DielectricPrepreg','DielectricOther','Mask','Other')),
  material_id INTEGER REFERENCES materials(object_id) ON DELETE RESTRICT,
  z_status TEXT NOT NULL CHECK(z_status IN ('Missing','Assumed','Confirmed')),
  z_num_le BLOB CHECK(z_num_le IS NULL OR length(z_num_le)=16),
  z_den_le BLOB CHECK(z_den_le IS NULL OR length(z_den_le)=16),
  thickness_status TEXT NOT NULL CHECK(thickness_status IN ('Missing','Assumed','Confirmed')),
  thickness_num_le BLOB CHECK(thickness_num_le IS NULL OR length(thickness_num_le)=16),
  thickness_den_le BLOB CHECK(thickness_den_le IS NULL OR length(thickness_den_le)=16),
  CHECK((z_status='Missing')=(z_num_le IS NULL AND z_den_le IS NULL)),
  CHECK((thickness_status='Missing')=(thickness_num_le IS NULL AND thickness_den_le IS NULL))
) STRICT;
-- Property<Rational> on each Layer has independent PropertySource evidence.
CREATE TABLE layer_property_sources (
  layer_id INTEGER NOT NULL REFERENCES layers(object_id) ON DELETE RESTRICT,
  property_key TEXT NOT NULL CHECK(property_key IN ('z_m','thickness_m')),
  document TEXT NOT NULL,
  revision TEXT NOT NULL,
  method TEXT NOT NULL,
  reference_frequency_hz REAL,
  temperature_c REAL,
  PRIMARY KEY(layer_id,property_key)
) STRICT, WITHOUT ROWID;
CREATE TABLE nets (
  object_id INTEGER PRIMARY KEY REFERENCES objects(id) ON DELETE RESTRICT,
  name TEXT NOT NULL,
  role TEXT NOT NULL CHECK(role IN ('Signal','Ground','Power','Floating','Unknown'))
) STRICT;
CREATE INDEX nets_name ON nets(name);

-- All point/analytic primitive bytes live in versioned geometry envelopes.
-- Relation columns are explicit; NULL means unverified/missing, never inferred.
CREATE TABLE geometry (
  object_id INTEGER PRIMARY KEY REFERENCES objects(id) ON DELETE RESTRICT,
  geometry_kind INTEGER NOT NULL CHECK(geometry_kind BETWEEN 1 AND 9),
  codec_version INTEGER NOT NULL CHECK(codec_version>=1),
  envelope BLOB NOT NULL CHECK(length(envelope)>=52),
  layer_id INTEGER REFERENCES layers(object_id) ON DELETE RESTRICT,
  net_id INTEGER REFERENCES nets(object_id) ON DELETE RESTRICT,
  parent_geometry_id INTEGER REFERENCES geometry(object_id) ON DELETE RESTRICT,
  padstack_id INTEGER REFERENCES objects(id) ON DELETE RESTRICT,
  first_copper_order INTEGER,
  last_copper_order INTEGER,
  plating TEXT CHECK(plating IN ('NonPlated','SourceDeclaredPlated','Unknown')),
  CHECK((first_copper_order IS NULL AND last_copper_order IS NULL) OR
    (first_copper_order IS NOT NULL AND last_copper_order IS NOT NULL
    AND first_copper_order>0 AND first_copper_order<last_copper_order))
) STRICT;
CREATE INDEX geometry_layer_net ON geometry(layer_id,net_id);
CREATE INDEX geometry_padstack ON geometry(padstack_id);
CREATE TABLE geometry_refs (
  geometry_id INTEGER NOT NULL REFERENCES geometry(object_id) ON DELETE RESTRICT,
  ref_index INTEGER NOT NULL CHECK(ref_index>=0),
  target_object_id INTEGER NOT NULL REFERENCES objects(id) ON DELETE RESTRICT,
  ref_role TEXT NOT NULL CHECK(ref_role IN
    ('Frame','Boundary','Hole','Padstack','Pad','Via','Drill','Component','Other')),
  PRIMARY KEY(geometry_id,ref_index)
) STRICT, WITHOUT ROWID;
CREATE INDEX geometry_refs_target ON geometry_refs(target_object_id);

CREATE TABLE components (
  object_id INTEGER PRIMARY KEY REFERENCES objects(id) ON DELETE RESTRICT,
  reference TEXT NOT NULL,
  placement_frame_id INTEGER NOT NULL REFERENCES coordinate_frames(object_id) ON DELETE RESTRICT,
  x_ticks INTEGER NOT NULL,
  y_ticks INTEGER NOT NULL,
  orientation TEXT NOT NULL CHECK(orientation IN ('Orthogonal','PendingRotation')),
  quarter_turn INTEGER CHECK(quarter_turn BETWEEN 0 AND 3),
  pending_radians REAL,
  mirror_x INTEGER NOT NULL CHECK(mirror_x IN (0,1)),
  bottom_side INTEGER NOT NULL CHECK(bottom_side IN (0,1)),
  CHECK((orientation='Orthogonal' AND quarter_turn IS NOT NULL AND pending_radians IS NULL)
     OR (orientation='PendingRotation' AND quarter_turn IS NULL AND pending_radians IS NOT NULL))
) STRICT;
CREATE TABLE pins (
  object_id INTEGER PRIMARY KEY REFERENCES objects(id) ON DELETE RESTRICT,
  component_id INTEGER NOT NULL REFERENCES components(object_id) ON DELETE RESTRICT,
  pin_number TEXT NOT NULL,
  pad_id INTEGER NOT NULL REFERENCES geometry(object_id) ON DELETE RESTRICT,
  net_id INTEGER REFERENCES nets(object_id) ON DELETE RESTRICT
) STRICT;
CREATE INDEX pins_component ON pins(component_id);

CREATE TABLE connectivity_nodes (
  object_id INTEGER PRIMARY KEY REFERENCES objects(id) ON DELETE RESTRICT,
  conductor_id INTEGER NOT NULL REFERENCES geometry(object_id) ON DELETE RESTRICT,
  layer_id INTEGER NOT NULL REFERENCES layers(object_id) ON DELETE RESTRICT,
  frame_id INTEGER NOT NULL REFERENCES coordinate_frames(object_id) ON DELETE RESTRICT,
  x_ticks INTEGER NOT NULL,
  y_ticks INTEGER NOT NULL,
  witness_kind TEXT NOT NULL CHECK(witness_kind IN
    ('TraceVertex','TraceInterior','ArcStart','ArcEnd','PadCentre',
     'ViaCentre','AreaWitness','AreaInterior')),
  trace_vertex_index INTEGER CHECK(trace_vertex_index IS NULL OR trace_vertex_index>=0),
  witness_evidence TEXT NOT NULL DEFAULT '',
  CHECK((witness_kind='TraceVertex')=(trace_vertex_index IS NOT NULL))
) STRICT;
CREATE TABLE connectivity_edges (
  object_id INTEGER PRIMARY KEY REFERENCES objects(id) ON DELETE RESTRICT,
  first_node INTEGER NOT NULL REFERENCES connectivity_nodes(object_id) ON DELETE RESTRICT,
  second_node INTEGER NOT NULL REFERENCES connectivity_nodes(object_id) ON DELETE RESTRICT,
  evidence_kind TEXT NOT NULL CHECK(evidence_kind IN
    ('SameLayerContact','ViaBarrel','AreaContinuity')),
  evidence_text TEXT NOT NULL,
  witness_object_id INTEGER REFERENCES objects(id) ON DELETE RESTRICT,
  verified_state TEXT NOT NULL CHECK(verified_state IN ('Unverified','Candidate','SourceDeclared','Verified')),
  CHECK(first_node<>second_node),
  CHECK((evidence_kind='SameLayerContact' AND length(evidence_text)>0 AND witness_object_id IS NULL)
    OR (evidence_kind IN ('ViaBarrel','AreaContinuity') AND witness_object_id IS NOT NULL))
) STRICT;
CREATE INDEX connectivity_edges_nodes ON connectivity_edges(first_node,second_node);

CREATE TABLE diagnostics (
  id INTEGER PRIMARY KEY,
  object_id INTEGER REFERENCES objects(id) ON DELETE RESTRICT,
  severity TEXT NOT NULL CHECK(severity IN ('Error','Missing','Assumption','Information')),
  code TEXT NOT NULL,
  message TEXT NOT NULL,
  source_evidence TEXT NOT NULL DEFAULT ''
) STRICT;
CREATE INDEX diagnostics_object ON diagnostics(object_id);

CREATE TABLE extensions (
  extension_name TEXT NOT NULL,
  extension_version INTEGER NOT NULL CHECK(extension_version>=1),
  required INTEGER NOT NULL CHECK(required IN (0,1)),
  payload BLOB NOT NULL,
  PRIMARY KEY(extension_name,extension_version)
) STRICT, WITHOUT ROWID;

-- Contract: the Reader MUST also perform semantic validation, not merely SQL checks.
-- In particular object.kind, geometry_kind, geometry_refs, rational signs/reduction,
-- required SourceFile ordering, material finite-ness and electrical proof.
